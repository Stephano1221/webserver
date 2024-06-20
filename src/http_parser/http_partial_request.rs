use super::{HttpRequest, HttpVersion};

#[derive(Clone)]
pub struct PartialHttpRequest<'a> {
    pub request: HttpRequest<'a>,
    pub next_byte: usize,
}

impl PartialHttpRequest<'_> {
    pub fn new() -> Self {
        PartialHttpRequest {
            request: HttpRequest::default(),
            next_byte: 0
        }
    }

    pub fn version(&self) -> &Option<HttpVersion> {
        &self.request.version
    }
}
