use core::fmt;

use super::{HttpHeader, HttpStatusCode, HttpVersion};

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
