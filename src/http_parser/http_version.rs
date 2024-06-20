use core::fmt;

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
