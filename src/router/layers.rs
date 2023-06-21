use crate::{
    router::Layer,
    service::{
        response::{ErrorResponse, Response, ResponseResult},
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

pub struct ErrorHeader(
    pub &'static str, /* System */
    pub &'static str, /* User */
);

impl Layer for ErrorHeader {
    fn map_response(&self, response: ResponseResult) -> ResponseResult {
        response.map_err(|response| match response {
            ErrorResponse::System(response) => ErrorResponse::System(Response {
                header: self.0.into(),
                ..response
            }),
            ErrorResponse::User(response) => ErrorResponse::User(Response {
                header: self.1.into(),
                ..response
            }),
        })
    }
}
