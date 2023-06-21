use std::fmt::Display;

pub use response_body::{Parts, ResponseBody};
pub use response_part::{Html, ResponsePart};

pub use crate::message::{Kind, Part};

pub mod response_part {
    use super::*;

    pub type ResponsePart = Part;

    impl<'a> From<&'a str> for ResponsePart {
        fn from(value: &'a str) -> Self {
            ResponsePart {
                kind: Kind::Text,
                content: value.as_bytes().into(),
            }
        }
    }

    impl From<String> for ResponsePart {
        fn from(value: String) -> Self {
            ResponsePart {
                kind: Kind::Text,
                content: value.as_bytes().into(),
            }
        }
    }

    pub struct Html(pub String);

    impl From<Html> for ResponsePart {
        fn from(value: Html) -> Self {
            ResponsePart {
                kind: Kind::Html,
                content: value.0.as_bytes().into(),
            }
        }
    }

    impl<'a, N: AsRef<str>> From<(N, &'a str)> for ResponsePart {
        fn from((name, content): (N, &'a str)) -> Self {
            ResponsePart {
                kind: Kind::Attachment(name.as_ref().into()),
                content: content.as_bytes().into(),
            }
        }
    }

    impl<N: AsRef<str>> From<(N, String)> for ResponsePart {
        fn from((name, content): (N, String)) -> Self {
            ResponsePart {
                kind: Kind::Attachment(name.as_ref().into()),
                content: content.as_bytes().into(),
            }
        }
    }

    impl<N: AsRef<str>> From<(N, Vec<u8>)> for ResponsePart {
        fn from((name, content): (N, Vec<u8>)) -> Self {
            ResponsePart {
                kind: Kind::Attachment(name.as_ref().into()),
                content: content.into(),
            }
        }
    }
}

pub mod response_body {
    use super::*;

    pub struct ResponseBody(pub Vec<Part>);

    impl ToString for ResponseBody {
        fn to_string(&self) -> String {
            self.0.iter().filter_map(|part| part.as_utf8().ok()).fold(
                String::new(),
                |mut acc, v| {
                    acc.push_str(v);
                    acc.push_str("\n\n");
                    acc
                },
            )
        }
    }

    impl From<()> for ResponseBody {
        fn from(_: ()) -> Self {
            ResponseBody(vec![])
        }
    }

    impl<P: Into<ResponsePart>> From<P> for ResponseBody {
        fn from(p: P) -> Self {
            ResponseBody(vec![p.into()])
        }
    }

    impl From<Vec<Part>> for ResponseBody {
        fn from(body: Vec<Part>) -> Self {
            ResponseBody(body)
        }
    }

    pub struct Parts<T>(pub T);

    impl<P1: Into<ResponsePart>, P2: Into<ResponsePart>> From<Parts<(P1, P2)>> for ResponseBody {
        fn from(Parts((p1, p2)): Parts<(P1, P2)>) -> Self {
            ResponseBody(vec![p1.into(), p2.into()])
        }
    }
}

pub struct Response {
    pub header: String,
    pub body: ResponseBody,
}

impl<T: Display> From<T> for Response {
    fn from(value: T) -> Self {
        Response {
            header: "".into(),
            body: value.to_string().into(),
        }
    }
}

pub enum ErrorResponse {
    System(Response),
    User(Response),
}

impl<E: Into<Response>> From<E> for ErrorResponse {
    fn from(error: E) -> Self {
        ErrorResponse::System(error.into())
    }
}

pub fn user_error<E: Into<Response>>(error: E) -> ErrorResponse {
    ErrorResponse::User(error.into())
}

pub type ResponseResult = Result<Option<Response>, ErrorResponse>;

impl Response {
    pub fn ok(header: impl Into<String>, body: impl Into<ResponseBody>) -> ResponseResult {
        Ok(Some(Response {
            header: header.into(),
            body: body.into(),
        }))
    }

    pub fn sys_err(header: impl Into<String>, body: impl Into<ResponseBody>) -> ResponseResult {
        Err(ErrorResponse::System(Response {
            header: header.into(),
            body: body.into(),
        }))
    }

    pub fn user_err(header: impl Into<String>, body: impl Into<ResponseBody>) -> ResponseResult {
        Err(ErrorResponse::User(Response {
            header: header.into(),
            body: body.into(),
        }))
    }

    pub fn none() -> ResponseResult {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responses() {
        let _ = Response::none();
        let _ = Response::sys_err("", "This is an error");
        let _ = Response::user_err("", "This is an error");
        let _ = Response::ok("", ());
        let _ = Response::ok("", "value");
        let _ = Response::ok("", ("name", "content"));
        let _ = Response::ok("", ("name", String::from("content")));
        let _ = Response::ok("", ("name", vec![0x65]));
        let _ = Response::ok("", vec!["value".into(), ("name", vec![0x65]).into()]);
        let _ = Response::ok("", Parts(("value", ("name", vec![0x65]))));
    }

    #[test]
    fn system_error_into_result() {
        fn foo() -> ResponseResult {
            Err("Error response")?;
            Response::none()
        }

        let _ = foo();
    }

    #[test]
    fn user_error_into_result() {
        fn foo() -> ResponseResult {
            Err("Error response").map_err(user_error)?;
            Response::none()
        }

        let _ = foo();
    }
}
