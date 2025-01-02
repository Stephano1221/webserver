use std::{collections::hash_map, io};

use crate::helper::{bytes, enums::Processing};

use super::{HttpFieldName, HttpHeader, HttpMethod, HttpStatusCode, HttpTarget, HttpVersion, PartialHttpRequest};

#[derive(Clone, Default, Debug, PartialEq)]
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
    /// Tries to parse an array of bytes into a [`HttpRequest`]. No parsing will be done on the body.
    /// 
    /// If all of the bytes for the request have been received, then it should return a
    /// [`Processing<Finished<Result<HttpRequest>>>`].
    /// 
    /// If only part of the request's bytes are provided, then it will parse what it can
    /// and should return a [`Processing<InProgress<())>>`], which indicates that the
    /// supplied `partial_request` can be passed back into this function to continue
    /// processing once more `request_bytes` are received.
    /// 
    /// TODO: The supplied `partial_request` will be modified in this method, and the returned
    /// references will either be `partial_request`'s [`PartialHttpRequest`] if there are
    /// more bytes to process, or the [`HttpRequest`] inside it if processing is finished.
    /// 
    /// # Bad Data
    /// If the request doesn't contain a full, understood request header (method, target
    /// and HTTP version), this function will return a [`Processing<Finished<Result<(Error, HttpStatusCode)>>>`]
    /// with a recommended [`HttpStatusCode`].
    /// 
    /// If field names are unknown, the field will be ignored.
    /// If field names or field values contain non-UTF8 characters, the entire field line will be ignored.
    pub fn try_parse<'a>(partial_request: &PartialHttpRequest<'a>, request_bytes: &'a [u8]) -> Processing<PartialHttpRequest<'a>, Result<HttpRequest<'a>, (io::Error, HttpStatusCode)>> {
        let mut partial_request = partial_request.clone();
        let word_delimiter = b" ";
        let line_delimiter = b"\r\n";
        let body_delimiter = b"\r\n\r\n";
        let bad_request = Processing::Finished(Err((io::ErrorKind::InvalidInput.into(), HttpStatusCode::BadRequest400)));
        let not_implemented = Processing::Finished(Err((io::ErrorKind::InvalidInput.into(), HttpStatusCode::NotImplemented501)));
        let version_not_supported = Processing::Finished(Err((io::ErrorKind::InvalidInput.into(), HttpStatusCode::HttpVersionNotSupported505)));

        // Method
        if partial_request.request.method.is_none() {
            partial_request.request.method = match Self::find_until(&mut partial_request, request_bytes, word_delimiter) {
                None => return Processing::InProgress(partial_request),
                Some(before_delimiter) => match std::str::from_utf8(before_delimiter) {
                    Err(_) => return bad_request,
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
                    Err(_) => return bad_request,
                    Ok(slice) => match HttpTarget::from_str(slice) {
                        None => return bad_request,
                        Some(target) => Some(target),
                    },
                },
            }
        }
        
        // Version
        if partial_request.request.version.is_none() {
            partial_request.request.version = match Self::find_until(&mut partial_request, request_bytes, line_delimiter) {
                None => return Processing::InProgress(partial_request),
                Some(before_delimiter) => match std::str::from_utf8(before_delimiter) {
                    Err(_) => return bad_request,
                    Ok(slice) => match HttpVersion::from_str(slice) {
                        None => return version_not_supported,
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
                    None => return bad_request,
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
                        hash_map::Entry::Occupied(entry) => {
                            match entry.get().parse::<usize>() {
                                Err(_) => return bad_request,
                                Ok(content_length) => {
                                    if content_length == 0 {
                                        None
                                    } else {
                                        let end_index = partial_request.next_byte + content_length;
                                        if request_bytes.len() < end_index {
                                            return Processing::InProgress(partial_request)
                                        }
                                        Some(&request_bytes[partial_request.next_byte..end_index])
                                    }
                                },
                            }
                        },
                    }
                },
            }
        }

        Processing::Finished(Ok(partial_request.request))
    }

    /// Returns the subdomain of the request, as determined by the `Host` header.
    /// 
    /// The first domain name in `domain_names` that is found in the `Host` header is used,
    /// so domain names should be sorted by descending order of length (specificity) to
    /// ensure that if there are two domains names, one with and one without a subdomain,
    /// the one with the subdomain will be matched against (thus not returning the matching subdomain).
    /// 
    /// Sorting of `domain_names` isn't done here for performance reasons. Note that this
    /// may change in the future.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use webserver::http_parser::HttpRequest;
    /// # use webserver::http_parser::HttpHeader;
    /// let mut header = HttpHeader::new();
    /// header.insert("Host", "uk.shop.example.com");
    /// let request = HttpRequest {
    /// #     method: None,
    /// #     target: None,
    /// #     version: None,
    ///      header: Some(header),
    /// #     body: None,
    /// };
    /// let domain_names = vec!("example.com");
    /// let subdomain = request.subdomain(domain_names);
    /// assert_eq!(subdomain, Some("uk.shop"));
    /// ```
    /// 
    /// ```
    /// 
    /// # use webserver::http_parser::HttpRequest;
    /// # use webserver::http_parser::HttpHeader;
    /// let mut header = HttpHeader::new();
    /// header.insert("Host", "uk.shop.example.com");
    /// # let request = HttpRequest {
    /// #     method: None,
    /// #     target: None,
    /// #     version: None,
    /// #     header: Some(header),
    /// #     body: None,
    /// };
    /// // The order of the domain names is important!
    /// let domain_names = vec!("example.com", "shop.example.com");
    /// let subdomain = request.subdomain(domain_names);
    /// assert_eq!(subdomain, Some("uk.shop"));
    /// let domain_names = vec!("shop.example.com", "example.com");
    /// let subdomain = request.subdomain(domain_names);
    /// assert_eq!(subdomain, Some("uk"));
    /// ```
    /// 
    /// ```
    /// 
    /// # use webserver::http_parser::HttpRequest;
    /// # use webserver::http_parser::HttpHeader;
    /// # let mut header = HttpHeader::new();
    /// header.insert("Host", "example.com");
    /// # let request = HttpRequest {
    /// #     method: None,
    /// #     target: None,
    /// #     version: None,
    /// #     header: Some(header),
    /// #     body: None,
    /// # };
    /// let domain_names = vec!("example.com");
    /// let subdomain = request.subdomain(domain_names);
    /// assert_eq!(subdomain, None);
    /// ```
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
            match host.rfind(domain_name) {
                None => continue,
                Some(index) => {
                    let subdomain = &host[..index];
                    if subdomain.ends_with(subdomain_delimiter) {
                        return Some(&subdomain[..(subdomain.len() - subdomain_delimiter.len_utf8())])
                    }
                    return None
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

#[cfg(test)]
mod tests {
    mod try_parse {
        use super::super::*;

        #[test]
        fn full_request() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 HTTP/1.1\r\nHost: example.com\r\nContent-Length:3\r\n\r\nabc";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_request = HttpRequest {
                method: Some(HttpMethod::Get),
                target: HttpTarget::from_str("/path1/path2"),
                version: Some(HttpVersion::Http1Dot1),
                header: HttpHeader::from_bytes(b"Host: example.com\r\nContent-Length: 3\r\n\r\n"),
                body: Some(b"abc"),
            };

            if let Processing::Finished(Ok(request)) = result {
                assert_eq!(request, expected_request);
            } else {
                panic!("Expected Processing::Finished(Ok(HttpRequest)), but got {:?}", result);
            }
        }

        #[test]
        fn partial_up_until_method() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET ";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_request = HttpRequest {
                method: Some(HttpMethod::Get),
                target: None,
                version: None,
                header: None,
                body: None,
            };
            let expected_partial_request = PartialHttpRequest {
                request: expected_request,
                next_byte: 4,
            };
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request, expected_partial_request);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_up_until_target() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 ";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_request = HttpRequest {
                method: Some(HttpMethod::Get),
                target: HttpTarget::from_str("/path1/path2"),
                version: None,
                header: None,
                body: None,
            };
            let expected_partial_request = PartialHttpRequest {
                request: expected_request,
                next_byte: 17,
            };
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request, expected_partial_request);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_up_until_http_version() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 HTTP/1.1\r\n";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_request = HttpRequest {
                method: Some(HttpMethod::Get),
                target: HttpTarget::from_str("/path1/path2"),
                version: Some(HttpVersion::Http1Dot1),
                header: None,
                body: None,
            };
            let expected_partial_request = PartialHttpRequest {
                request: expected_request,
                next_byte: 27,
            };
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request, expected_partial_request);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }
        
        #[test]
        fn partial_up_until_headers() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 HTTP/1.1\r\nHost: example.com\r\nContent-Length: 3\r\n\r\n";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_request = HttpRequest {
                method: Some(HttpMethod::Get),
                target: HttpTarget::from_str("/path1/path2"),
                version: Some(HttpVersion::Http1Dot1),
                header: HttpHeader::from_bytes(b"Host: example.com\r\nContent-Length: 3\r\n\r\n"),
                body: None,
            };
            let expected_partial_request = PartialHttpRequest {
                request: expected_request,
                next_byte: 67,
            };
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request, expected_partial_request);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_method() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_method = None;
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request.request.method, expected_method);
                assert_eq!(partial_request.next_byte, 0);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_path() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_target = None;
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request.request.target, expected_target);
                assert_eq!(partial_request.next_byte, 4);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_http_version() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 HTTP/1.1";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_version= None;
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request.request.version, expected_version);
                assert_eq!(partial_request.next_byte, 17);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_headers() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 HTTP/1.1\r\nHost: example.com\r\nContent-Length: 3\r\n\r";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_headers = None;
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request.request.header, expected_headers);
                assert_eq!(partial_request.next_byte, 27);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn partial_body() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET /path1/path2 HTTP/1.1\r\nHost: example.com\r\nContent-Length: 3\r\n\r\nab";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_body = None;
    
            if let Processing::InProgress(partial_request) = result {
                assert_eq!(partial_request.request.body, expected_body);
                assert_eq!(partial_request.next_byte, 67);
            } else {
                panic!("Expected Processing::InProgress(PartialHttpRequest), but got {:?}", result);
            }
        }

        #[test]
        fn invalid_method() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"HELLO ";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);

            let expected_error = io::ErrorKind::InvalidInput;
            let expected_status_code = HttpStatusCode::NotImplemented501;
    
            if let Processing::Finished(Err(err )) = result {
                let error = err.0.kind();
                let status_code = err.1;
                assert_eq!(error, expected_error);
                assert_eq!(status_code, expected_status_code);
            } else {
                panic!("Expected Processing::Finished(Err(Error, HttpStatusCode)), but got {:?}", result);
            }
        }

        #[test]
        fn invalid_target() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET  ";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);
            let expected_error = io::ErrorKind::InvalidInput;
            let expected_status_code = HttpStatusCode::BadRequest400;
    
            if let Processing::Finished(Err(err )) = result {
                let error = err.0.kind();
                let status_code = err.1;
                assert_eq!(error, expected_error);
                assert_eq!(status_code, expected_status_code);
            } else {
                panic!("Expected Processing::Finished(Err(Error, HttpStatusCode)), but got {:?}", result);
            }
        }

        #[test]
        fn invalid_version() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / ABC/1.1\r\n";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);
            let expected_error = io::ErrorKind::InvalidInput;
            let expected_status_code = HttpStatusCode::HttpVersionNotSupported505;
    
            if let Processing::Finished(Err(err )) = result {
                let error = err.0.kind();
                let status_code = err.1;
                assert_eq!(error, expected_error);
                assert_eq!(status_code, expected_status_code);
            } else {
                panic!("Expected Processing::Finished(Err(Error, HttpStatusCode)), but got {:?}", result);
            }
        }

        #[test]
        fn invalid_header() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / HTTP/1.1\r\nHost: example.com\r\ninvalid\r\nContent-Length: 0\r\n\r\n";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);
            let expected_error = io::ErrorKind::InvalidInput;
            let expected_status_code = HttpStatusCode::BadRequest400;
    
            if let Processing::Finished(Err(err )) = result {
                let error = err.0.kind();
                let status_code = err.1;
                assert_eq!(error, expected_error);
                assert_eq!(status_code, expected_status_code);
            } else {
                panic!("Expected Processing::Finished(Err(Error, HttpStatusCode)), but got {:?}", result);
            }
        }

        #[test]
        fn missing_header() {
            let partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / HTTP/1.1\r\n\r\n\r\n";

            let result = HttpRequest::try_parse(&partial_request, request_bytes);
            let expected_error = io::ErrorKind::InvalidInput;
            let expected_status_code = HttpStatusCode::BadRequest400;
    
            if let Processing::Finished(Err(err )) = result {
                let error = err.0.kind();
                let status_code = err.1;
                assert_eq!(error, expected_error);
                assert_eq!(status_code, expected_status_code);
            } else {
                panic!("Expected Processing::Finished(Err(Error, HttpStatusCode)), but got {:?}", result);
            }
        }
    }
    mod subdomain {
        use super::super::*;

        #[test]
        fn with_subdomain() {
            let domain_names = vec!("example.com");
            let mut header = HttpHeader::new();
            header.insert("Host", "uk.shop.example.com");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = Some("uk.shop");
    
            assert_eq!(result, expected_result);
        }

        #[test]
        fn subdomain_partial_match() {
            let domain_names = vec!("ample.com");
            let mut header = HttpHeader::new();
            header.insert("Host", "uk.shop.example.com");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }

        #[test]
        fn subdomain_double_match() {
            let domain_names = vec!("example.com");
            let mut header = HttpHeader::new();
            header.insert("Host", "example.com.example.com");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = Some("example.com");
    
            assert_eq!(result, expected_result);
        }

        #[test]
        fn subdomain_dot() {
            let domain_names = vec!("example.com");
            let mut header = HttpHeader::new();
            header.insert("Host", ".");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }
    
        #[test]
        fn exact_match() {
            let domain_names = vec!("example.com");
            let mut header = HttpHeader::new();
            header.insert("Host", "example.com");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }

        #[test]
        fn host_header_is_none() {
            let domain_names = vec!("");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: None,
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }
    
        #[test]
        fn newly_created_host_header() {
            let domain_names = vec!("example.com");
            let header = HttpHeader::new();
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }

        #[test]
        fn empty_subdomain() {
            let domain_names = vec!("");
            let mut header = HttpHeader::new();
            header.insert("Host", "");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }
    
        #[test]
        fn empty_host() {
            let domain_names = vec!("example.com");
            let mut header = HttpHeader::new();
            header.insert("Host", "");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }
    
        #[test]
        fn no_domain_names() {
            let domain_names = vec!();
            let mut header = HttpHeader::new();
            header.insert("Host", "example.com");
            let request = HttpRequest {
                method: None,
                target: None,
                version: None,
                header: Some(header),
                body: None,
            };
    
            let result = request.subdomain(domain_names);
            let expected_result = None;
    
            assert_eq!(result, expected_result);
        }
    }
    mod find_until {
        use super::super::*;

        #[test]
        fn first_space() {
            let mut partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / HTTP/1.1\r\n";
            let delimiter = b" ";

            let result = HttpRequest::find_until(&mut partial_request, request_bytes, delimiter);
            let expected_result = Some(&request_bytes[..3]);

            assert_eq!(result, expected_result);
        }

        #[test]
        fn second_space() {
            let mut partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / HTTP/1.1\r\n";
            let delimiter = b" ";

            HttpRequest::find_until(&mut partial_request, request_bytes, delimiter);

            let result = HttpRequest::find_until(&mut partial_request, request_bytes, delimiter);
            let expected_result = Some(&request_bytes[4..=4]);

            assert_eq!(result, expected_result);
        }

        #[test]
        fn not_found() {
            let mut partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / HTTP/1.1\r\n";
            let delimiter = b"Nonexistant";

            let result = HttpRequest::find_until(&mut partial_request, request_bytes, delimiter);
            let expected_result = None;

            assert_eq!(result, expected_result);
        }

        #[test]
        fn empty_request_bytes() {
            let mut partial_request = PartialHttpRequest::new();
            let request_bytes = b"";
            let delimiter = b"Nonexistant";

            let result = HttpRequest::find_until(&mut partial_request, request_bytes, delimiter);
            let expected_result = None;

            assert_eq!(result, expected_result);
        }

        #[test]
        fn empty_delimiter() {
            let mut partial_request = PartialHttpRequest::new();
            let request_bytes = b"GET / HTTP/1.1\r\n";
            let delimiter = b"";

            let result = HttpRequest::find_until(&mut partial_request, request_bytes, delimiter);
            let expected_result = Some(&request_bytes[..0]);

            assert_eq!(result, expected_result);
        }
    }
}