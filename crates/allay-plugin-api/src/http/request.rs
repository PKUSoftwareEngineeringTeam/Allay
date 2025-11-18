use crate::component::export::Request;
use crate::component::export::{Header, Method};

impl Request {
    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    pub fn method(&self) -> &Method {
        &self.ty
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    pub fn headers(&self) -> &[Header] {
        &self.headers
    }
}
