use core::fmt;
use std::{collections::{HashMap, hash_map}, io};

use crate::helper;

pub enum Processing<P, F> {
    InProgress(P),
    Finished(F)
}

#[derive(Clone)]
pub enum HttpMethod {
    Get,
    Head
}

impl HttpMethod {
    pub fn from_str(method: &str) -> Option<Self> {
        // Methods are case-sensitive
        match method.trim() {
            "GET" => Some(HttpMethod::Get),
            _ => None
        }
    }
}

#[derive(Clone)]
pub enum HttpVersion {
    Http1Dot1
}

impl HttpVersion {
    pub fn from_str(version: &str) -> Option<Self> {
        match version.trim() {
            "HTTP/1.1" => Some(HttpVersion::Http1Dot1),
            _ => None
        }
    }
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match self {
            HttpVersion::Http1Dot1 => "HTTP/1.1",
        };
        write!(f, "{version}")
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum HttpFieldName {
    ContentLength,
    Host
}

impl HttpFieldName {
    pub fn from_str(field_name: &str) -> Option<Self> {
        // As field names are case-insensitive, `field_name` and the cases below must be the same case
        match field_name.trim().to_ascii_lowercase().as_str() {
            "host" => Some(Self::Host),
            "content-length" => Some(Self::ContentLength),
            _ => None
        }
    }
}

impl fmt::Display for HttpFieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_name = match self {
            HttpFieldName::Host => "Host",
            HttpFieldName::ContentLength => "Content-Length"
        };
        write!(f, "{field_name}")
    }
}

#[derive(Clone)]
pub struct HttpHeader(HashMap<HttpFieldName, String>);

impl HttpHeader {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut fields = HashMap::new();
        let field_name_delimiter = b":";
        let line_delimiter = b"\r\n";
        let mut unprocessed_text = &bytes[..];
        while unprocessed_text.len() > 0 {
            let field_name_index = match helper::bytes::find(unprocessed_text, field_name_delimiter) {
                None => break,
                Some(index) => index,
            };
            let field_name = &unprocessed_text[..field_name_index];
            let new_start_index = if field_name_index >= unprocessed_text.len() { unprocessed_text.len() } else { field_name_index + 1 };
            unprocessed_text = &unprocessed_text[new_start_index..];

            let field_value_index = match helper::bytes::find(unprocessed_text, line_delimiter) {
                None => unprocessed_text.len(),
                Some(index) => index,
            };
            let field_value = &unprocessed_text[..field_value_index];
            let new_start_index = if field_value_index >= unprocessed_text.len() { unprocessed_text.len() } else { field_value_index + 1 };
            unprocessed_text = &unprocessed_text[new_start_index..];

            let field_name = match std::str::from_utf8(field_name) {
                Err(_) => continue,
                Ok(field_name) => {
                    match HttpFieldName::from_str(field_name) {
                        None => continue,
                        Some(field_name) => field_name,
                    }
                }
            };
            let field_value = match std::str::from_utf8(field_value) {
                Err(_) => continue,
                Ok(field_value) => field_value,
            };

            match fields.entry(field_name) {
                hash_map::Entry::Vacant(entry) => { entry.insert(field_value.to_owned()); },
                hash_map::Entry::Occupied(mut entry) => {
                    let old_value = entry.get();
                    let new_value = format!("{old_value}, {field_value}");
                    entry.insert(new_value);
                },
            };
        }
        match fields.len() {
            0 => None,
            _ => Some(HttpHeader(fields)),
        }
    }
}

impl fmt::Display for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        for (key, value) in &self.0 {
            output.push_str(&format!("{key}: {value}"));
        }
        write!(f, "{output}")
    }
}

#[derive(Clone)]
pub enum HttpStatusCode {
    OK200,
    BadRequest400,
    NotFound404,
    NotImplemented501,
}

impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            HttpStatusCode::OK200 => "200 OK",
            HttpStatusCode::BadRequest400 => "400 BAD REQUEST",
            HttpStatusCode::NotFound404 => "404 NOT FOUND",
            HttpStatusCode::NotImplemented501 => "501 NOT IMPLEMENTED",
        };
        write!(f, "{text}")
    }
}

#[derive(Clone)]
pub struct PartialHttpRequest<'a> {
    pub request: HttpRequest<'a>,
    next_byte: usize,
}

impl PartialHttpRequest<'_> {
    pub fn new() -> Self {
        PartialHttpRequest {
            request: HttpRequest::default(),
            next_byte: 0
        }
    }
}

#[derive(Clone, Default)]
pub struct HttpRequest<'a> {
    pub method: Option<HttpMethod>,
    pub target: Option<&'a str>,
    pub version: Option<HttpVersion>,
    pub header: Option<HttpHeader>,
    pub body: Option<&'a [u8]>
}

impl HttpRequest<'_> {
    /// NOTE: Some of this is wrong until the function is improved to use references properly.
    /// 
    /// Tries to parse an array of bytes into a [`HttpRequest`].
    /// 
    /// If all of the bytes for the request have been received, then it should return a
    /// [`Processing<Finished<Result<HttpRequest>>>`].
    /// 
    /// If only part of the request's bytes are provided, then it will parse what it can
    /// and should return a [`Processing<InProgress<())>>`], which indicates that the
    /// supplied `partial_request` can be passed back into this function to continue
    /// processing once more `request_bytes` are received.
    /// 
    /// The supplied `partial_request` will be modified in this method, and the returned
    /// references will either be `partial_request`'s [`PartialHttpRequest`] if there are
    /// more bytes to process, or the [`HttpRequest`] inside it if processing is finished.
    /// 
    /// # Bad Data
    /// If the request doesn't contain a full, understood request header (method, target
    /// and HTTP version), this function will return a [`Processing<Finished<Result<(Error, HttpStatusCode)>>>`]
    /// with a recommended [`HttpStatusCode`]. If field names are unknown, the field will be ignored.
    /// If field names or field values contain non-UTF8 characters, the entire field line will be ignored.
    /// No parsing will be done on the body.
    pub fn try_parse<'a>(partial_request: &PartialHttpRequest<'a>, request_bytes: &'a [u8]) -> Processing<PartialHttpRequest<'a>, Result<HttpRequest<'a>, (io::Error, HttpStatusCode)>> {
        let mut partial_request = partial_request.clone();
        let word_delimiter = b" ";
        let line_delimiter = b"\r\n";
        let body_delimiter = b"\r\n\r\n";
        let bad_request = Processing::Finished(Err((io::ErrorKind::InvalidInput.into(), HttpStatusCode::NotImplemented501)));
        let not_implemented = Processing::Finished(Err((io::ErrorKind::InvalidInput.into(), HttpStatusCode::NotImplemented501)));

        // Method
        if partial_request.request.method.is_none() {
            partial_request.request.method = match Self::find_until(&mut partial_request, request_bytes, word_delimiter) {
                None => return Processing::InProgress(partial_request),
                Some(before_delimiter) => match std::str::from_utf8(before_delimiter) {
                    Err(_) => return not_implemented,
                    Ok(slice) => match HttpMethod::from_str(slice) {
                        None => return not_implemented,
                        Some(method) => Some(method),
                    },
                },
            }
        }

        // Target
        if partial_request.request.target.is_none() {
            partial_request.request.target = match Self::find_until(&mut partial_request, request_bytes, word_delimiter) {
                None => return Processing::InProgress(partial_request),
                Some(before_delimiter) => match std::str::from_utf8(before_delimiter) {
                    Err(_) => return not_implemented,
                    Ok(slice) => Some(slice),
                },
            }
        }
        
        // Version
        if partial_request.request.version.is_none() {
            partial_request.request.version = match Self::find_until(&mut partial_request, request_bytes, line_delimiter) {
                None => return Processing::InProgress(partial_request),
                Some(before_delimiter) => match std::str::from_utf8(before_delimiter) {
                    Err(_) => return not_implemented,
                    Ok(slice) => match HttpVersion::from_str(slice) {
                        None => return not_implemented,
                        Some(method) => Some(method),
                    },
                },
            }
        }

        // Header
        if partial_request.request.header.is_none() {
            partial_request.request.header = match Self::find_until(&mut partial_request, request_bytes, body_delimiter) {
                None => return Processing::InProgress(partial_request),
                Some(before_delimiter) => match HttpHeader::from_bytes(before_delimiter) {
                    None => None,
                    Some(header) => (Some(header)),
                },
            }
        }

        //Body
        if partial_request.request.body.is_none() {
            partial_request.request.body = match &mut partial_request.request.header {
                None => None,
                Some(header) => {
                    match header.0.entry(HttpFieldName::ContentLength) {
                        hash_map::Entry::Vacant(_) => None,
                        hash_map::Entry::Occupied(length) => {
                            match length.get().parse() {
                                Err(_) => return bad_request,
                                Ok(end_index) => {
                                    Some(&request_bytes[partial_request.next_byte..end_index])
                                },
                            }
                        },
                    }
                },
            }
        }

        Processing::Finished(Ok(partial_request.request))
    }

    fn find_until<'a>(partial_request: &mut PartialHttpRequest, request_bytes: &'a [u8], delimiter: &[u8]) -> Option<&'a [u8]> {
        let start_index = partial_request.next_byte;
        let unprocessed_bytes = &request_bytes[start_index..];
        match helper::bytes::find(unprocessed_bytes, delimiter) {
            None => None,
            Some(index) => {
                let end_index = start_index + index;
                partial_request.next_byte = end_index + delimiter.len();
                Some(&request_bytes[start_index..end_index])
            },
        }
    }
}

pub struct HttpResponse<'a> {
    pub version: HttpVersion,
    pub status_code: HttpStatusCode,
    pub header: Option<HttpHeader>,
    pub body: Option<&'a [u8]>
}

// impl HttpResponse<'_> {
//     pub fn new<'a>(version: HttpVersion, status_code: HttpStatusCode, header: Option<HttpHeader>, body: Option<&[u8]>) -> Self {
//         HttpResponse {
//             version,
//             status_code,
//             header,
//             body
//         }
//     }
// }

impl fmt::Display for HttpResponse<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = match &self.header {
            Some(header) => header.to_string(),
            None => String::new(),
        };
        // TODO: Write body
        // let body = match self.body {
        //     Some(body) => std::str::from_utf8(body).unwrap(),
        //     None => "",
        // };
        write!(f, "{} {}\r\n{}\r\n\r\n", self.version, self.status_code, header)
    }
}