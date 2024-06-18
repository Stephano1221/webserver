use std::{error::Error, fs::OpenOptions, io::{self, Read}, net::SocketAddr};

use crate::http_parser::{HttpFieldName, HttpHeader, HttpMethod, HttpRequest, HttpResponse, HttpStatusCode, HttpTarget, HttpVersion};

pub struct Config {
    pub content_directory: String,
    pub root_directory: String,
    pub subdomain_directory: String,
    pub socket: SocketAddr,
    pub request_default_filename: String,
}

pub fn get_response<'a>(config: &Config, http_request: &'a mut Result<HttpRequest, (io::Error, HttpStatusCode)>) -> (Option<HttpResponse>) {
    match http_request {
        Err((error, status_code)) => {
            (Some(HttpResponse::new(&HttpVersion::Http1Dot1, &status_code, &None, &None)))
        }
        Ok(request) => {
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
                Err((response, error)) => (Some(response)),
                Ok(response) => (Some(response)),
            }
        }
    }
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
    let mut header = http_response.header.as_mut().expect("`http_response.header` should be `Some`");
    header.insert(HttpFieldName::ContentLength.to_string().as_str(), bytes.to_string().as_str());
    http_response.body = Some(body);
    Ok(http_response)
    // Err((HttpResponse::new(http_version, &HttpStatusCode::InternalServerError500, &None, &None), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_head(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    format_directory(config, http_request);
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
    // Err((HttpResponse::new(&http_request.version.as_ref().expect("`http_request.version` should be `Some`"), &HttpStatusCode::InternalServerError500, &None, &None), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn http_post(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    not_implemented(config, http_request)
}

fn http_put(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    not_implemented(config, http_request)
}

fn http_delete(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    not_implemented(config, http_request)
}

fn http_connect(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    not_implemented(config, http_request)
}

fn http_options(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    not_implemented(config, http_request)
}

fn http_trace(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    not_implemented(config, http_request)
}

fn not_implemented(config: &Config, http_request: &mut HttpRequest) -> Result<HttpResponse, (HttpResponse, Box<dyn Error>)> {
    let http_version = http_request.version.as_ref().expect("`http_request.version` should be `Some`");
    Err((HttpResponse::new(http_version, &HttpStatusCode::NotImplemented501, &None, &None), Box::new(io::Error::new(io::ErrorKind::Other, ""))))
}

fn format_directory(config: &Config, http_request: &mut HttpRequest) {
    let target = match &http_request.target {
        None => &HttpTarget::new(),
        Some(target) => target,
    };
    let mut path = match &target.path {
        None => String::new(),
        Some(path) => path.to_owned(),
    };
    let prefix = format!("{}/{}", config.content_directory, config.root_directory);
    path.insert_str(0, &prefix);
    http_request.target.as_mut().expect("`http_request.target` should be `Some`").path = Some(path);
}

fn set_filename(http_request: &mut HttpRequest, filename: &str) {
    if http_request.target.is_none() {
        http_request.target = Some(HttpTarget::new());
    }
    http_request.target.as_mut().expect("`http_request.target` should be `Some`").set_filename(filename);
}

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