use std::{collections::hash_map, io};

use crate::helper::{bytes, enums::Processing};

use super::{HttpFieldName, HttpHeader, HttpMethod, HttpStatusCode, HttpTarget, HttpVersion, PartialHttpRequest};

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
                    Some(header) => Some(header),
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

    pub fn subdomain(&self, domain_names: Vec<&str>) -> Option<&str> {
        if let None = self.header {
            return None
        }
        let header = self.header.as_ref().expect("`self.header` should be `Some`");
        let host = match header.get_value(HttpFieldName::Host.to_string().as_str()) {
            None => return None,
            Some(host) => host,
        };
        let subdomain_delimiter = '.';
        for domain_name in domain_names {
            match host.find(domain_name) {
                None => continue,
                Some(index) => {
                    return if index > 0 {
                        let subdomain = &host[..index];
                        if subdomain.ends_with(subdomain_delimiter) {
                            Some(&subdomain[..(subdomain.len() - subdomain_delimiter.len_utf8())])
                        } else {
                            Some(subdomain)
                        }
                    } else {
                        None
                    }
                },
            }
        }
        None
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
