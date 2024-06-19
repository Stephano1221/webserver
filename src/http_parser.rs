use core::fmt;
use std::{collections::{hash_map, HashMap}, io::{self, BufWriter, Read}};

use crate::helper::{bytes, enums::Processing};

pub enum HttpProtocol {
    Http,
    Https
}

impl HttpProtocol {
    pub fn port(&self) -> u16 {
        match self {
            HttpProtocol::Http => 80,
            HttpProtocol::Https => 443
        }
    }
}

#[derive(Clone, Debug)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> Option<Self> {
        // Methods are case-sensitive
        match method.trim() {
            "GET" => Some(HttpMethod::Get),
            "HEAD" => Some(HttpMethod::Head),
            "POST" => Some(HttpMethod::Post),
            "PUT" => Some(HttpMethod::Put),
            "DELETE" => Some(HttpMethod::Delete),
            "CONNECT" => Some(HttpMethod::Connect),
            "OPTIONS" => Some(HttpMethod::Options),
            "TRACE" => Some(HttpMethod::Trace),
            _ => None
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Connect => "CONNECT",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
        };
        write!(f, "{method}")
    }
}

#[derive(Clone, Debug)]
pub enum HttpVersion {
    Http1Dot1
}

impl HttpVersion {
    pub fn from_str(version: &str) -> Option<Self> {
        match version.trim() {
            "HTTP/1.1" => Some(Self::Http1Dot1),
            _ => None
        }
    }
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match self {
            Self::Http1Dot1 => "HTTP/1.1",
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
            Self::Host => "Host",
            Self::ContentLength => "Content-Length"
        };
        write!(f, "{field_name}")
    }
}

#[derive(Clone, Debug)]
pub struct HttpHeader(HashMap<String, String>);

impl HttpHeader {
    pub fn new() -> Self {
        HttpHeader(HashMap::new())
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut fields = HashMap::new();
        let field_name_delimiter = b":";
        let line_delimiter = b"\r\n";
        let mut unprocessed_bytes = &bytes[..];
        while unprocessed_bytes.len() > 0 {
            let field_name_separator_index = match bytes::find(unprocessed_bytes, field_name_delimiter) {
                None => break,
                Some(index) => index,
            };
            let field_name = &unprocessed_bytes[..field_name_separator_index];
            let new_start_index = if field_name_separator_index >= unprocessed_bytes.len() { unprocessed_bytes.len() } else { field_name_separator_index + field_name_delimiter.len() };
            unprocessed_bytes = &unprocessed_bytes[new_start_index..];

            let field_value_separator_index = match bytes::find(unprocessed_bytes, line_delimiter) {
                None => unprocessed_bytes.len(),
                Some(index) => index,
            };
            let field_value = &unprocessed_bytes[..field_value_separator_index];
            let new_start_index = if field_value_separator_index >= unprocessed_bytes.len() { unprocessed_bytes.len() } else { field_value_separator_index + line_delimiter.len() };
            unprocessed_bytes = &unprocessed_bytes[new_start_index..];

            let field_name = match std::str::from_utf8(field_name) {
                Err(_) => continue,
                Ok(field_name) => field_name.trim(),
            };
            let field_value = match std::str::from_utf8(field_value) {
                Err(_) => continue,
                Ok(field_value) => field_value.trim(),
            };

            match fields.entry(field_name.to_owned()) {
                hash_map::Entry::Vacant(entry) => { entry.insert(field_value.to_owned()); },
                hash_map::Entry::Occupied(mut entry) => {
                    // let old_value = entry.get();
                    // let new_value = format!("{old_value}, {field_value}");
                    // entry.insert(new_value);

                    // let old_value = entry.get_mut();
                    // *old_value = format!("{old_value}, {field_value}");

                    *entry.get_mut() = format!("{}, {field_value}", entry.get());
                },
            };
        }
        match fields.len() {
            0 => None,
            _ => Some(HttpHeader(fields)),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.insert(key.to_owned(), value.to_owned());
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
    Continue100,
    SwitchingProtocols101,
    OK200,
    Created201,
    Accepted202,
    NonAuthoritativeInformation203,
    NoContent204,
    ResetContent205,
    PartialContent206,
    MultipleChoices300,
    MovedPermanently301,
    Found302,
    SeeOther303,
    NotModified304,
    UseProxy305,
    TemporaryRedirect307,
    PermanentRedirect308,
    BadRequest400,
    Unauthorized401,
    PaymentRequired402,
    Forbidden403,
    NotFound404,
    MethodNowAllowed405,
    NotAcceptable406,
    ProxyAuthenticationRequired407,
    RequestTimeout408,
    Conflict409,
    Gone410,
    LengthRequired411,
    PreconditionFailed412,
    ContentTooLarge413,
    UriTooLong414,
    UnsupportedMediaType415,
    RangeNotSatisfiable416,
    ExpectationFailed417,
    MisdirectedRequest421,
    UnprocessableContent422,
    UpgradeRequired426,
    InternalServerError500,
    NotImplemented501,
    BadGateway502,
    ServiceUnavailable503,
    GatewayTimeout504,
    HttpVersionNotSupported505,
}

impl HttpStatusCode {
    pub fn from_io_error(error: &io::Error) -> Self {
        // Errors commented out below are unstable.
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound404,
            io::ErrorKind::PermissionDenied => Self::Forbidden403,
            io::ErrorKind::ConnectionRefused => Self::BadGateway502,
            io::ErrorKind::ConnectionReset => Self::BadGateway502,
            //io::ErrorKind::HostUnreachable => todo!(),
            //io::ErrorKind::NetworkUnreachable => todo!(),
            io::ErrorKind::ConnectionAborted => Self::BadGateway502,
            io::ErrorKind::NotConnected => Self::GatewayTimeout504,
            io::ErrorKind::AddrInUse => Self::InternalServerError500,
            io::ErrorKind::AddrNotAvailable => Self::InternalServerError500,
            //io::ErrorKind::NetworkDown => todo!(),
            io::ErrorKind::BrokenPipe => Self::InternalServerError500,
            io::ErrorKind::AlreadyExists => Self::Conflict409,
            io::ErrorKind::WouldBlock => Self::InternalServerError500,
            //io::ErrorKind::NotADirectory => todo!(),
            //io::ErrorKind::IsADirectory => todo!(),
            //io::ErrorKind::DirectoryNotEmpty => todo!(),
            //io::ErrorKind::ReadOnlyFilesystem => todo!(),
            //io::ErrorKind::FilesystemLoop => todo!(),
            //io::ErrorKind::StaleNetworkFileHandle => todo!(),
            io::ErrorKind::InvalidInput => Self::InternalServerError500,
            io::ErrorKind::InvalidData => Self::InternalServerError500,
            io::ErrorKind::TimedOut => Self::InternalServerError500,
            io::ErrorKind::WriteZero => Self::InternalServerError500,
            //io::ErrorKind::StorageFull => todo!(),
            //io::ErrorKind::NotSeekable => todo!(),
            //io::ErrorKind::FilesystemQuotaExceeded => todo!(),
            //io::ErrorKind::FileTooLarge => todo!(),
            //io::ErrorKind::ResourceBusy => todo!(),
            //io::ErrorKind::ExecutableFileBusy => todo!(),
            //io::ErrorKind::Deadlock => todo!(),
            //io::ErrorKind::CrossesDevices => todo!(),
            //io::ErrorKind::TooManyLinks => todo!(),
            //io::ErrorKind::InvalidFilename => todo!(),
            //io::ErrorKind::ArgumentListTooLong => todo!(),
            io::ErrorKind::Interrupted => Self::InternalServerError500,
            io::ErrorKind::Unsupported => Self::InternalServerError500,
            io::ErrorKind::UnexpectedEof => Self::InternalServerError500,
            io::ErrorKind::OutOfMemory => Self::InternalServerError500,
            io::ErrorKind::Other => Self::InternalServerError500,
            _ => Self::InternalServerError500,
        }
    }
}

impl fmt::Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            HttpStatusCode::Continue100 => "100 CONTINUE",
            HttpStatusCode::SwitchingProtocols101 => "101 SWITCHING PROTOCOLS",
            HttpStatusCode::OK200 => "200 OK",
            HttpStatusCode::Created201 => "201 CREATED",
            HttpStatusCode::Accepted202 => "202 ACCEPTED",
            HttpStatusCode::NonAuthoritativeInformation203 => "203 NON AUTHORITATIVE INFORMATION",
            HttpStatusCode::NoContent204 => "204 NO CONTENT",
            HttpStatusCode::ResetContent205 => "205 RESET CONTENT",
            HttpStatusCode::PartialContent206 => "206 PARTIAL CONTENT",
            HttpStatusCode::MultipleChoices300 => "300 MULTIPLE CHOICES",
            HttpStatusCode::MovedPermanently301 => "301 MOVED PERMANENTLY",
            HttpStatusCode::Found302 => "302 FOUND",
            HttpStatusCode::SeeOther303 => "303 SEE OTHER",
            HttpStatusCode::NotModified304 => "304 NOT MODIFIED",
            HttpStatusCode::UseProxy305 => "305 USE PROXY",
            HttpStatusCode::TemporaryRedirect307 => "307 TEMPORARY REDIRECT",
            HttpStatusCode::PermanentRedirect308 => "308 PERMANENT REDIRECT",
            HttpStatusCode::BadRequest400 => "400 BAD REQUEST",
            HttpStatusCode::Unauthorized401 => "401 UNAUTHORIZED",
            HttpStatusCode::PaymentRequired402 => "402 PAYMENT REQUIRED",
            HttpStatusCode::Forbidden403 => "403 FORBIDDEN",
            HttpStatusCode::NotFound404 => "404 NOT FOUND",
            HttpStatusCode::MethodNowAllowed405 => "405 METHOD NOW ALLOWED",
            HttpStatusCode::NotAcceptable406 => "406 NOT ACCEPTABLE",
            HttpStatusCode::ProxyAuthenticationRequired407 => "407 PROXY AUTHENTICATION REQUIRED",
            HttpStatusCode::RequestTimeout408 => "408 REQUEST TIMEOUT",
            HttpStatusCode::Conflict409 => "409 CONFLICT",
            HttpStatusCode::Gone410 => "410 GONE",
            HttpStatusCode::LengthRequired411 => "411 LENGTH REQUIRED",
            HttpStatusCode::PreconditionFailed412 => "412 PRECONDITION FAILED",
            HttpStatusCode::ContentTooLarge413 => "413 CONTENT TOO LARGE",
            HttpStatusCode::UriTooLong414 => "414 URI TOO LONG",
            HttpStatusCode::UnsupportedMediaType415 => "415 UNSUPPORTED MEDIA TYPE",
            HttpStatusCode::RangeNotSatisfiable416 => "416 RANGE NOT SATISFIABLE",
            HttpStatusCode::ExpectationFailed417 => "417 EXPECTATION FAILED",
            HttpStatusCode::MisdirectedRequest421 => "421 MISDIRECTED REQUEST",
            HttpStatusCode::UnprocessableContent422 => "422 UNPROCESSABLE CONTENT",
            HttpStatusCode::UpgradeRequired426 => "426 UPGRADE REQUIRED",
            HttpStatusCode::InternalServerError500 => "500 INTERNAL SERVER ERROR",
            HttpStatusCode::NotImplemented501 => "501 NOT IMPLEMENTED",
            HttpStatusCode::BadGateway502 => "502 BAD GATEWAY",
            HttpStatusCode::ServiceUnavailable503 => "503 SERVICE UNAVAILABLE",
            HttpStatusCode::GatewayTimeout504 => "504 GATEWAY TIMEOUT",
            HttpStatusCode::HttpVersionNotSupported505 => "505 HTTP VERSION NOT SUPPORTED",
        };
        write!(f, "{text}")
    }
}

// #[derive(Clone)]
// pub struct Filepath {
//     pub directory: Option<String>,
//     pub filename: Option<String>,
// }

// impl Filepath {
//     pub fn empty() -> Self {
//         Filepath {
//             directory: None,
//             filename: None,
//         }
//     }

//     pub fn from_str(slice: &str) -> Result<Self, ()> {
//         let directory_delimiter = '/';
//         let file_extension_delimiter = '.';
//         match slice.rfind(directory_delimiter) {
//             None => Err(()),
//             Some(directory_index) => {
//                 if directory_index == slice.len() - 1 {
//                     return Ok(Filepath {
//                         directory: Some(slice.to_owned()),
//                         filename: None,
//                     })
//                 }
//                 match slice[(directory_index + 1)..].find(file_extension_delimiter) {
//                     None => Ok(Filepath {
//                         directory: Some(slice.to_owned()),
//                         filename: None,
//                     }),
//                     Some(_file_extension_index) => Ok(Filepath {
//                         directory: Some(slice[..=directory_index].to_owned()),
//                         filename: Some(slice[(directory_index + 1)..].to_owned()),
//                     }),
//                 }
//             },
//         }
//     }
// }

// impl fmt::Display for Filepath {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let directory = match &self.directory{
//             None => "",
//             Some(directory) => directory,
//         };
//         let filename = match &self.filename{
//             None => "",
//             Some(filename) => filename,
//         };
//         write!(f, "{}{}", directory, filename)
//     }
// }

#[derive(Clone, Debug)]
pub struct TargetParameters(HashMap<String, Vec<String>>);

impl TargetParameters {
    pub fn from_str(target: &str) -> Result<Self, ()> {
        let mut parameters = HashMap::new();
        let query_delimiter = '?';
        let target = match target.find(query_delimiter) {
            None => &target[..],
            Some(index) => &target[(index + 1)..],
        };
        let key_delimiter = '=';
        let parameter_delimiter = '&';
        let mut unprocessed_text = &target[..];
        while unprocessed_text.len() > 0 {
            let key_separator_index = match unprocessed_text.find(key_delimiter) {
                None => break,
                Some(index) => index,
            };
            let key = &unprocessed_text[..key_separator_index];
            let new_start_index = if key_separator_index >= unprocessed_text.len() { unprocessed_text.len() } else { key_separator_index + 1 };
            unprocessed_text = &unprocessed_text[new_start_index..];

            let parameter_separator_index = match unprocessed_text.find(parameter_delimiter) {
                None => unprocessed_text.len(),
                Some(index) => index,
            };
            let value = &unprocessed_text[..parameter_separator_index];
            let new_start_index = if parameter_separator_index >= unprocessed_text.len() { unprocessed_text.len() } else { parameter_separator_index + 1 };
            unprocessed_text = &unprocessed_text[new_start_index..];

            match parameters.entry(key.to_owned()) {
                hash_map::Entry::Vacant(entry) => {
                    let mut values = Vec::new();
                    values.push(value.to_owned());
                    entry.insert(values);
                },
                hash_map::Entry::Occupied(mut entry) => {
                    let values = entry.get_mut();
                    values.push(value.to_owned());
                },
            }
        }
        match parameters.len() {
            0 => Err(()),
            _ => Ok(Self(parameters)),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.0.get(key)
    }
}

#[derive(Clone, Default, Debug)]
pub struct HttpTarget {
    pub path: Option<String>,
    pub parameters: Option<TargetParameters>,
}

impl HttpTarget {
    /// Creates a new, empty (containing `None`) `HttpTarget`.
    pub fn new() -> Self {
        HttpTarget {
            path: None,
            parameters: None,
        }
    }

    pub fn from_str(target: &str) -> Result<Self, ()> {
        let parameter_delimiter = '?';
        // let filepath = match Filepath::from_str(target) {
        //     Err(_) => None,
        //     Ok(path) => Some(path),
        // };
        let path = match target.split_once(parameter_delimiter) {
            None => Some(target.to_owned()),
            Some((path, _)) => Some(path.to_owned()),
        };
        let parameters = match TargetParameters::from_str(target) {
            Err(_) => None,
            Ok(parameters) => Some(parameters),
        };
        if !path.is_none() || !parameters.is_none() {
            Ok(HttpTarget {
                path,
                parameters,
            })
        } else {
            Err(())
        }
    }

    pub fn directory(&self) -> Option<&str> {
        self.get_directory_and_filename().0
    }

    pub fn filename(&self) -> Option<&str> {
        self.get_directory_and_filename().1
    }

    /// Sets the `path`'s filename.
    /// 
    /// # Safety
    /// The new `path` will be `Some`.
    pub fn set_filename(&mut self, filename: &str) {
        let directory_delimiter = '/';
        let old_directory = match self.directory() {
            None => Some(directory_delimiter.to_string()),
            Some(directory) => Some(directory.to_owned()),
        };
        let mut new_path = old_directory.expect("`old_directory` should be `Some`");
        if !new_path.ends_with(directory_delimiter) {
            new_path.push(directory_delimiter);
        }
        new_path.push_str(filename);
        self.path = Some(new_path);
    }

    /// Sets the `path`'s directory.
    /// 
    /// # Safety
    /// The new `path` will be `Some`.
    pub fn set_directory(&mut self, directory: &str) {
        let filename = self.filename();
        let directory_delimiter = '/';
        let mut new_path = directory.to_owned();
        if filename.is_some() {
            if !new_path.ends_with(directory_delimiter) {
                new_path.push(directory_delimiter);
            }
            new_path.push_str(filename.expect("`filename` should be `Some`"));
        }
        self.path = Some(new_path);
    }

    pub fn n_directories(&self, directories: usize) -> Option<&str> {
        if let None = self.path {
            return None
        }
        let directory_delimiter = '/';
        let full_path = &self.path.as_ref().expect("`path` should be `Some`")[..];
        let mut unprocessed_path = &full_path[..];
        let mut found_directories = 0;
        let mut last_directory_separator_index = 0;
        while unprocessed_path.len() > 0 {
            match unprocessed_path.find(directory_delimiter) {
                None => return None,
                Some(index) => {
                    found_directories += 1;
                    last_directory_separator_index += index;
                    if found_directories >= directories {
                        return Some(&full_path[..=last_directory_separator_index])
                    }
                    let new_start_index = if last_directory_separator_index >= unprocessed_path.len() { unprocessed_path.len() } else { last_directory_separator_index + 1 };
                    unprocessed_path = &unprocessed_path[new_start_index..];
                },
            }
        };
        None
    }

    fn get_directory_and_filename(&self) -> (Option<&str>, Option<&str>) {
        let directory_delimiter = '/';
        let filename_extension_delimiter = '.';
        match &self.path {
            None => (None, None),
            Some(path) => {
                match path.rfind(filename_extension_delimiter) {
                    None => (Some(path), None),
                    Some(_) => {
                        match path.rfind(directory_delimiter) {
                            None => (None, Some(path)),
                            Some(index) => (Some(&path[..=index]), Some(&path[(index + 1)..])),
                        }
                    },
                }
            },
        }
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

#[derive(Clone, Default, Debug)]
pub struct HttpRequest<'a> {
    pub method: Option<HttpMethod>,
    pub target: Option<HttpTarget>,
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
                    Ok(slice) => match HttpTarget::from_str(slice) {
                        Err(_) => return not_implemented,
                        Ok(target) => Some(target),
                    },
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
                    match header.0.entry(HttpFieldName::ContentLength.to_string()) {
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

    // pub fn get_target_filepath(&self) -> Filepath {
    //     let file_path = match &self.target {
    //         None => return Filepath::empty(),
    //         Some(target) => {
    //             if target.len() <= 0 {
    //                 return Filepath::empty()
    //             }
    //             match target.find('?') {
    //                 None => self.target.expect("`self.target` should be `Some`"),
    //                 Some(index) => self.target.expect("`self.target` should be `Some`")[..index],
    //             }
    //         },
    //     }.as_str();

    //     let directory_delimiter = '/';
    //     let (directory, file_name) = match file_path.rfind(directory_delimiter) {
    //         None => ("", file_path),
    //         Some(index) => {
    //             let file_name_start_index = if index >= file_path.len() - 1 { index } else { index + 1 };
    //             (&file_path[..=index], &file_path[(file_name_start_index)..])
    //         },
    //     };

    //     Filepath {
    //         directory: directory.to_owned(),
    //         filename: file_name.to_owned(),
    //     }
    // }

    fn find_until<'a>(partial_request: &mut PartialHttpRequest, request_bytes: &'a [u8], delimiter: &[u8]) -> Option<&'a [u8]> {
        let start_index = partial_request.next_byte;
        let unprocessed_bytes = &request_bytes[start_index..];
        match bytes::find(unprocessed_bytes, delimiter) {
            None => None,
            Some(index) => {
                let end_index = start_index + index;
                partial_request.next_byte = end_index + delimiter.len();
                Some(&request_bytes[start_index..end_index])
            },
        }
    }
}

pub struct HttpResponse {
    pub version: HttpVersion,
    pub status_code: HttpStatusCode,
    pub header: Option<HttpHeader>,
    pub body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn new(version: &HttpVersion, status_code: &HttpStatusCode, header: &Option<&HttpHeader>, body: &Option<&[u8]>) -> Self {
        HttpResponse {
            version: version.clone(),
            status_code: status_code.clone(),
            header: header.cloned(),
            body: body.map(|bytes| bytes.to_vec()),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.version.to_string().as_bytes());
        bytes.extend_from_slice(&[b' ']);
        bytes.extend_from_slice(self.status_code.to_string().as_bytes());
        bytes.extend_from_slice(&[b'\r', b'\n']);
        if let Some(header) = &self.header {
            bytes.extend_from_slice(header.to_string().as_bytes());
        }
        bytes.extend_from_slice(&[b'\r', b'\n', b'\r', b'\n']);
        if let Some(body) = &self.body {
            bytes.extend_from_slice(body);
        }
        bytes
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = match &self.header {
            Some(header) => header.to_string(),
            None => String::new(),
        };
        write!(f, "{} {}\r\n{}\r\n\r\n[Body]", self.version, self.status_code, header)
    }
}