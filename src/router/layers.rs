use crate::{
    router::Layer,
    service::{
        response::{Response, ResponseResult},
        Request,
    },
};

pub struct LowercaseHeader;

impl Layer for LowercaseHeader {
    fn map_request(&self, request: Request) -> Request {
        Request {
            header: request.header.to_lowercase(),
            ..request
        }
    }
}

pub struct ErrorHeader(pub &'static str);

impl Layer for ErrorHeader {
    fn map_response(&self, response: ResponseResult) -> ResponseResult {
        response.map_err(|response| Response {
            header: self.0.into(),
            ..response
        })
    }
}
