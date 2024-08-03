use std::{
    error::Error,
    fs::OpenOptions,
    io::{self, Read},
    net::TcpStream
};

use crate::{
    config::Config,
    http_parser::{HttpFieldName, HttpHeader, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode, HttpTarget, HttpVersion},
    network
};

/// Starts the server with the specified configuration
pub fn start_server(config: &Config) {
    network::start_listener(config);
}

/// Handles a HTTP request
pub fn handle_request(config: &Config, stream: &mut TcpStream, http_request: &mut Result<HttpRequest, (io::Error, HttpStatusCode)>) {
    let http_response = get_response(config, http_request);
    match &http_response {
        None => return,
        Some(response) => {
            match send_response(stream, &response) {
                Err(_error) => todo!(),
                Ok(()) => return,
            }
        },
    }
}

/// Gets a response to a HTTP request
pub fn get_response<'a>(config: &Config, http_request: &'a mut Result<HttpRequest, (io::Error, HttpStatusCode)>) -> Option<HttpResponse> {
    match http_request {
        Err((error, status_code)) => { Some(HttpResponse::new(&HttpVersion::Http1Dot1, &status_code, &None, &None)) }
        Ok(request) => {
            println!("{:#?}", request);
            let method = request.method.as_ref().expect("`request.method` should be `Some`");
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
                Err((mut response, error)) => {
                    match response.status_code {
                        HttpStatusCode::NotFound404 => set_body_not_found(config, request, &mut response),
                        _ => ()
                    }
                    Some(response)
                },
                Ok(response) => Some(response),
            }
        }
    }
}

/// Sends a [`HttpResponse`] to the specified `stream`.
pub fn send_response(stream: &mut TcpStream, http_response: &HttpResponse) -> Result<(), Box<dyn Error>> {
    println!("Response: {}", http_response.to_string());
    network::send_bytes(stream, &http_response.as_bytes())?;
    Ok(())
}

fn http_get(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    let mut http_response = http_head(config, http_request)?;
    let http_version = http_request.version.as_ref().expect("`http_request.version` should be `Some`");
    let path = http_request.target.as_ref().expect("`http_request.target` should be `Some`").path.as_ref().expect("`http_request.target.path` should be `Some`");

    let mut file = match OpenOptions::new().read(true).open(path) {
        Err(error) => return Err((HttpResponse::new(http_version, &HttpStatusCode::from_io_error(&error), &None, &None), Box::new(error))),
        Ok(file) => file,
    };

    let mut body = vec!();
    let bytes = match file.read_to_end(&mut body) {
        Err(error) => return Err((HttpResponse::new(http_version, &HttpStatusCode::from_io_error(&error), &None, &None), Box::new(error))),
        Ok(bytes) => bytes,
    };

    if http_response.header.is_none() {
        http_response.header = Some(HttpHeader::new());
    };

    let header = http_response.header.as_mut().expect("`http_response.header` should be `Some`");
    header.insert(HttpFieldName::ContentLength.to_string().as_str(), bytes.to_string().as_str());
    http_response.body = Some(body);
    Ok(http_response)
}

fn http_head(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    add_target_prefix(config, http_request);
    set_filename_if_none(http_request, &config.request_default_filename);
    let http_version = http_request.version.as_ref().expect("`http_request.version` should be `Some`");

    let path = http_request.target.as_ref().expect("`http_request.target` should be `Some`").path.as_ref().expect("`http_request.target.path` should be `Some`");
    let file = match OpenOptions::new().read(true).open(path) {
        Err(error) => return Err((HttpResponse::new(http_version, &HttpStatusCode::from_io_error(&error), &None, &None), Box::new(error))),
        Ok(file) => file,
    };

    let metadata = match file.metadata() {
        Err(error) => return Err((HttpResponse::new(http_version, &HttpStatusCode::from_io_error(&error), &None, &None), Box::new(error))),
        Ok(metadata) => metadata,
    };

    let mut http_header = HttpHeader::new();
    http_header.insert(HttpFieldName::ContentLength.to_string().as_str(), metadata.len().to_string().as_str());

    Ok(HttpResponse {
        version: http_version.clone(),
        status_code: HttpStatusCode::OK200,
        header: Some(http_header),
        body: None,
    })
}

fn http_post(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((not_implemented_response(http_request), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_put(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((not_implemented_response(http_request), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_delete(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((not_implemented_response(http_request), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_connect(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((not_implemented_response(http_request), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_options(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((not_implemented_response(http_request), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_trace(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    Err((not_implemented_response(http_request), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

/// Gets the response for when the method in a HTTP request is not recognised/implemented by the server.
fn not_implemented_response(http_request: &mut HttpRequest) -> HttpResponse {
    let http_version = http_request.version.as_ref().expect("`http_request.version` should be `Some`");
    HttpResponse::new(http_version, &HttpStatusCode::NotImplemented501, &None, &None)
}

/// Sets the body of the `http_response` to the relevant 'not-found' file, if one is found.
fn set_body_not_found(config: &Config, http_request: &HttpRequest, http_response: &mut HttpResponse) {
    let not_found_path = get_not_found_path(config, http_request);
    let mut file = match OpenOptions::new().read(true).open(not_found_path) {
        Err(error) => return,
        Ok(file) => file,
    };

    let mut body = vec!();
    let bytes = match file.read_to_end(&mut body) {
        Err(error) => return,
        Ok(bytes) => bytes,
    };

    if http_response.header.is_none() {
        http_response.header = Some(HttpHeader::new());
    };

    let header = http_response.header.as_mut().expect("`http_response.header` should be `Some`");
    header.insert(HttpFieldName::ContentLength.to_string().as_str(), bytes.to_string().as_str());

    http_response.body = Some(body);
}

/// Gets the filepath of the file to display when the requested file cannot be found.
/// 
/// This is on a per subdomain basis, with each full subdomain having its own 'not-found' file.
fn get_not_found_path(config: &Config, http_request: &HttpRequest) -> String {
    let directory_delimiter = '/';
    let mut path = get_target_prefix(config, http_request);

    if !path.ends_with(directory_delimiter) {
        path.push(directory_delimiter);
    }
    path.push_str(&config.not_found_filename);
    path
}

/// Adds the directory prefix for the specified root or subdomain(s) to the target
/// to make it the full target path.
fn add_target_prefix(config: &Config, http_request: &mut HttpRequest) {
    let prefix = get_target_prefix(config, http_request);
    if let None = http_request.target {
        http_request.target = Some(HttpTarget::new());
    }
    let target = http_request.target.as_mut().expect("`http_request.target` should be `Some`");

    let mut path = match &target.path {
        None => String::new(),
        Some(path) => path.to_owned(),
    };

    path.insert_str(0, &prefix);
    target.path = Some(path);
}

/// Get the directory prefix for the specified root or subdomain(s) that can be prefixed to the target
/// to get the full target path.
fn get_target_prefix(config: &Config, http_request: &HttpRequest) -> String {
    match http_request.subdomain(config.domain_names.iter().map(|s| s.as_ref()).collect()) {
        None => format!("{}/{}", config.top_directory, config.root_directory),
        Some(subdomain) => format!("{}/{}/{}", config.top_directory, config.subdomain_directory, subdomain_as_path(subdomain)),
    }
}

/// Gets the subdomain as a directory path, where each subdomain is a deeper folder.
/// 
/// The subdomains are read right-to-left.
/// 
/// # Examples
/// 
/// ```
/// let subdomain = "uk.shop";
/// let path = subdomain_as_path(subdomain);
/// assert_eq!("shop/uk", path);
/// ```
/// 
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
    http_request.target.as_mut().expect("`http_request.target` should be `Some`").set_filename(filename);
}

/// Sets the target filename for a [`HttpRequest`] only if it doesn't already have one.
fn set_filename_if_none(http_request: &mut HttpRequest, filename: &str) {
    match &http_request.target {
        None => { set_filename(http_request, filename); },
        Some(target) => {
            if target.filename().is_none() {
                set_filename(http_request, filename);
            }
        },
    }
}
