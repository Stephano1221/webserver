use std::{
    error::Error,
    fs::OpenOptions,
    io::{self, Read},
    net::TcpStream,
    path::{self},
    time::Instant,
};

use crate::{
    config::Config,
    http_parser::{
        HttpFieldName, HttpHeader, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode,
        HttpTarget, HttpVersion,
    },
    network,
};

/// Starts the server with the specified configuration
pub fn start_server(config: &Config) -> Result<(), Box<dyn Error>> {
    network::start_listener(config)?;
    Ok(())
}

/// Handles a HTTP request
pub fn handle_request(
    config: &Config,
    stream: &mut TcpStream,
    http_request: &mut Result<HttpRequest, (io::Error, HttpStatusCode)>,
    time_request_started: Instant,
) {
    let http_response = get_response(config, http_request);
    match &http_response {
        None => (),
        Some(response) => match send_response(config, stream, &response, time_request_started) {
            Err(error) => eprintln!("Error sending response: {}.", error),
            Ok(_) => (),
        },
    }
}

/// Gets a response to a HTTP request
pub fn get_response<'a>(
    config: &Config,
    http_request: &'a mut Result<HttpRequest, (io::Error, HttpStatusCode)>,
) -> Option<HttpResponse> {
    match http_request {
        Err((error, status_code)) => {
            eprintln!("Error getting response: {}.", error);
            Some(HttpResponse::new(
                &HttpVersion::Http1Dot1,
                &status_code,
                &None,
                &None,
            ))
        }
        Ok(request) => {
            println!("{:#?}", request);
            if let Some(response) = request_setup(config, request) {
                return Some(response);
            }
            let method = request.method.as_ref().unwrap();
            let result: Result<HttpResponse, (HttpResponse, Box<dyn Error>)> = match method {
                HttpMethod::Get => http_get(config, request),
                HttpMethod::Head => http_head(config, request),
                HttpMethod::Post => http_post(config, request),
                HttpMethod::Put => http_put(config, request),
                HttpMethod::Delete => http_delete(config, request),
                HttpMethod::Connect => http_connect(config, request),
                HttpMethod::Options => http_options(config, request),
                HttpMethod::Trace => http_trace(config, request),
            };
            match result {
                Err((mut response, _error)) => {
                    match response.status_code {
                        HttpStatusCode::NotFound404 => {
                            set_body_not_found(config, request, &mut response)
                        }
                        _ => (),
                    }
                    Some(response)
                }
                Ok(response) => Some(response),
            }
        }
    }
}

/// Sends a [`HttpResponse`] to the specified `stream`.
pub fn send_response(
    config: &Config,
    stream: &mut TcpStream,
    http_response: &HttpResponse,
    time_request_started: Instant,
) -> Result<(), Box<dyn Error>> {
    println!("Response: {}", http_response.to_string());
    network::send_bytes(
        config,
        stream,
        &http_response.as_bytes(),
        time_request_started,
    )?;
    Ok(())
}

fn request_setup(config: &Config, http_request: &mut HttpRequest) -> Option<HttpResponse> {
    add_target_prefix(config, http_request);
    set_filename_if_none(http_request, &config.global.default_filename);
    if is_a_directory_traversal_attack(http_request) {
        println!("Prevented directory traversal attack.");
        let mut response = HttpResponse::new(
            &HttpVersion::Http1Dot1,
            &HttpStatusCode::NotFound404,
            &None,
            &None,
        );
        set_body_not_found(config, http_request, &mut response);
        return Some(response);
    }
    None
}

fn is_a_directory_traversal_attack(http_request: &HttpRequest) -> bool {
    let path = http_request.target.as_ref().unwrap().path.as_ref().unwrap();
    // This could also be done by ensuring that the canonical path starts with the
    // expected target prefix path, but this would incur another filesystem call, and
    // potentially cause issues with soft links (symlinks), but not hard links.
    // A directory whitelist would mitigate the symlink issue.
    path.contains("..")
}

fn http_get(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    let mut http_response = http_head(config, http_request)?;
    let http_version = http_request.version.as_ref().unwrap();
    let path = http_request.target.as_ref().unwrap().path.as_ref().unwrap();

    let mut file = match OpenOptions::new().read(true).open(path) {
        Err(error) => {
            return Err((
                HttpResponse::new(
                    http_version,
                    &HttpStatusCode::from_io_error(&error),
                    &None,
                    &None,
                ),
                Box::new(error),
            ));
        }
        Ok(file) => file,
    };

    let mut body = vec![];
    let bytes = match file.read_to_end(&mut body) {
        Err(error) => {
            return Err((
                HttpResponse::new(
                    http_version,
                    &HttpStatusCode::from_io_error(&error),
                    &None,
                    &None,
                ),
                Box::new(error),
            ));
        }
        Ok(bytes) => bytes,
    };

    if http_response.header.is_none() {
        http_response.header = Some(HttpHeader::new());
    };

    let header = http_response.header.as_mut().unwrap();
    header.insert(
        HttpFieldName::ContentLength.to_string().as_str(),
        bytes.to_string().as_str(),
    );
    http_response.body = Some(body);
    Ok(http_response)
}

fn http_head(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    let http_version = http_request.version.as_ref().unwrap();

    let path = http_request.target.as_ref().unwrap().path.as_ref().unwrap();
    let file = match OpenOptions::new().read(true).open(path) {
        Err(error) => {
            return Err((
                HttpResponse::new(
                    http_version,
                    &HttpStatusCode::from_io_error(&error),
                    &None,
                    &None,
                ),
                Box::new(error),
            ));
        }
        Ok(file) => file,
    };

    let metadata = match file.metadata() {
        Err(error) => {
            return Err((
                HttpResponse::new(
                    http_version,
                    &HttpStatusCode::from_io_error(&error),
                    &None,
                    &None,
                ),
                Box::new(error),
            ));
        }
        Ok(metadata) => metadata,
    };

    let mut http_header = HttpHeader::new();
    http_header.insert(
        HttpFieldName::ContentLength.to_string().as_str(),
        metadata.len().to_string().as_str(),
    );
    if path.ends_with(".js") || path.ends_with(".mjs") {
        http_header.insert(
            HttpFieldName::ContentType.to_string().as_str(),
            "text/javascript",
        );
    }

    Ok(HttpResponse {
        version: http_version.clone(),
        status_code: HttpStatusCode::OK200,
        header: Some(http_header),
        body: None,
    })
}

fn http_post(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((
        not_implemented_response(http_request),
        Box::new(io::Error::new(io::ErrorKind::Other, "")),
    ))
}

fn http_put(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((
        not_implemented_response(http_request),
        Box::new(io::Error::new(io::ErrorKind::Other, "")),
    ))
}

fn http_delete(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((
        not_implemented_response(http_request),
        Box::new(io::Error::new(io::ErrorKind::Other, "")),
    ))
}

fn http_connect(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((
        not_implemented_response(http_request),
        Box::new(io::Error::new(io::ErrorKind::Other, "")),
    ))
}

fn http_options(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((
        not_implemented_response(http_request),
        Box::new(io::Error::new(io::ErrorKind::Other, "")),
    ))
}

fn http_trace(
    config: &Config,
    http_request: &mut HttpRequest,
) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((
        not_implemented_response(http_request),
        Box::new(io::Error::new(io::ErrorKind::Other, "")),
    ))
}

/// Gets the response for when the method in a HTTP request is not recognised/implemented by the server.
fn not_implemented_response(http_request: &mut HttpRequest) -> HttpResponse {
    let http_version = http_request.version.as_ref().unwrap();
    HttpResponse::new(
        http_version,
        &HttpStatusCode::NotImplemented501,
        &None,
        &None,
    )
}

/// Sets the body of the `http_response` to the relevant 'not-found' file, if one is found.
fn set_body_not_found(
    config: &Config,
    http_request: &HttpRequest,
    http_response: &mut HttpResponse,
) {
    let not_found_path = get_not_found_path(config, http_request);
    let mut file = match OpenOptions::new().read(true).open(not_found_path) {
        Err(error) => return,
        Ok(file) => file,
    };

    let mut body = vec![];
    let bytes = match file.read_to_end(&mut body) {
        Err(error) => return,
        Ok(bytes) => bytes,
    };

    if http_response.header.is_none() {
        http_response.header = Some(HttpHeader::new());
    };

    let header = http_response.header.as_mut().unwrap();
    header.insert(
        HttpFieldName::ContentLength.to_string().as_str(),
        bytes.to_string().as_str(),
    );

    http_response.body = Some(body);
}

/// Gets the filepath of the file to display when the requested file cannot be found.
///
/// This is on a per subdomain basis, with each full subdomain having its own 'not-found' file.
fn get_not_found_path(config: &Config, http_request: &HttpRequest) -> String {
    let directory_delimiter = std::path::MAIN_SEPARATOR;
    let mut path = get_target_prefix(config, http_request);

    if !path.ends_with(directory_delimiter) {
        path.push(directory_delimiter);
    }
    path.push_str(&config.global.not_found_filename);
    path
}

/// Adds the directory prefix for the specified root or subdomain(s) to the target.
///
/// This can either be absolute or relative, depending on `parent_directory` in `config`.
fn add_target_prefix(config: &Config, http_request: &mut HttpRequest) {
    let prefix = get_target_prefix(config, http_request);
    if http_request.target.is_none() {
        http_request.target = Some(HttpTarget::new());
    }
    let target = http_request.target.as_mut().unwrap();

    let mut path = match &target.path {
        None => String::new(),
        Some(path) => path.to_owned(),
    };

    path.insert_str(0, &prefix);
    target.path = Some(path);
}

/// Get the directory prefix for the specified root or subdomain(s), which can then be prefixed to the target.
///
/// This can either be absolute or relative, depending on `parent_directory` in `config`.
fn get_target_prefix(config: &Config, http_request: &HttpRequest) -> String {
    let directory_delimiter = std::path::MAIN_SEPARATOR;
    match http_request.subdomain(
        config
            .global
            .primary_domain_names
            .as_ref()
            .map(|s| s.iter().map(|s| s.as_str()).collect()),
    ) {
        None => format!(
            "{}{}{}",
            config.global.parent_directory,
            directory_delimiter,
            config.global.primary_domain_folder_name
        ),
        Some(subdomain) => format!(
            "{}{}{}{}{}",
            config.global.parent_directory,
            directory_delimiter,
            config.global.subdomains_folder_name,
            directory_delimiter,
            subdomain_as_path(subdomain)
        ),
    }
}

/// Gets the subdomain as a directory path, where each subdomain is a deeper folder.
///
/// The subdomains are read right-to-left.
///
/// # Examples
///
/// ```ignore
/// let subdomain = "uk.shop";
/// let path = subdomain_as_path(subdomain);
/// assert_eq!(path, "shop/uk");
/// ```
fn subdomain_as_path(subdomain: &str) -> String {
    let subdomain_delimiter = '.';
    let path_separator = '/';
    let subdomains = subdomain.rsplit(subdomain_delimiter);
    let mut path = String::with_capacity(subdomain.len());
    for subdomain in subdomains {
        path.push_str(subdomain);
        path.push(path_separator);
    }
    path.pop();
    path
}

/// Sets the target filename for a [`HttpRequest`], replacing any existing filename.
fn set_filename(http_request: &mut HttpRequest, filename: &str) {
    if http_request.target.is_none() {
        http_request.target = Some(HttpTarget::new());
    }
    http_request.target.as_mut().unwrap().set_filename(filename);
}

/// Sets the target filename for a [`HttpRequest`] only if it doesn't already have one.
fn set_filename_if_none(http_request: &mut HttpRequest, filename: &str) {
    match &http_request.target {
        None => {
            set_filename(http_request, filename);
        }
        Some(target) => {
            if target.filename().is_none() {
                set_filename(http_request, filename);
            }
        }
    }
}
