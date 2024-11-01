use core::fmt;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum HttpFieldName {
    ContentLength,
    ContentType,
    Host,
}

impl HttpFieldName {
    pub fn from_str(field_name: &str) -> Option<Self> {
        // As field names are case-insensitive, `field_name` and the cases below must be the same case
        match field_name.trim().to_ascii_lowercase().as_str() {
            "content-length" => Some(Self::ContentLength),
            "content-type" => Some(Self::ContentType),
            "host" => Some(Self::Host),
            _ => None
        }
    }
}

impl fmt::Display for HttpFieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field_name = match self {
            Self::ContentLength => "Content-Length",
            Self::ContentType => "Content-Type",
            Self::Host => "Host",
        };
        write!(f, "{field_name}")
    }
}
