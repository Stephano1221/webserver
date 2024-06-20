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
