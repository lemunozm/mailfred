use std::{fmt::Display, future::Future};

use async_trait::async_trait;
pub use response::{Body, Cancel, Empty, Error, Response, ResponseResult};
pub use response_part::ResponsePart;

use crate::transport::{Kind, Message, Part};

pub type Request = Message;
#[async_trait]
pub trait Service: Send + Clone + 'static {
    type Response: Into<Response>;

    async fn process(self, req: Request) -> Self::Response;
}

#[async_trait]
impl<F, Fut, Res> Service for F
where
    F: FnOnce(Message) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Res> + Send,
    Res: Into<Response>,
{
    type Response = Res;

    async fn process(self, req: Request) -> Self::Response {
        (self)(req).await
    }
}

pub mod response_part {
    use super::*;

    pub type ResponsePart = Part;

    impl<'a> From<&'a str> for ResponsePart {
        fn from(value: &'a str) -> ResponsePart {
            ResponsePart {
                kind: Kind::Text,
                content: value.as_bytes().into(),
            }
        }
    }

    impl From<String> for ResponsePart {
        fn from(value: String) -> ResponsePart {
            ResponsePart {
                kind: Kind::Text,
                content: value.as_bytes().into(),
            }
        }
    }

    impl<'a, N: AsRef<str>> From<(N, &'a str)> for ResponsePart {
        fn from((name, content): (N, &'a str)) -> ResponsePart {
            ResponsePart {
                kind: Kind::Attachment(name.as_ref().into()),
                content: content.as_bytes().into(),
            }
        }
    }

    impl<N: AsRef<str>> From<(N, String)> for ResponsePart {
        fn from((name, content): (N, String)) -> ResponsePart {
            ResponsePart {
                kind: Kind::Attachment(name.as_ref().into()),
                content: content.as_bytes().into(),
            }
        }
    }

    impl<N: AsRef<str>> From<(N, Vec<u8>)> for ResponsePart {
        fn from((name, content): (N, Vec<u8>)) -> ResponsePart {
            ResponsePart {
                kind: Kind::Attachment(name.as_ref().into()),
                content: content.into(),
            }
        }
    }
}

pub mod response {
    use super::*;

    pub struct Response(pub Option<Result<Vec<Part>, Vec<Part>>>);
    pub type ResponseResult<R> = Result<R, Box<dyn std::error::Error>>;

    /// Type to indicate will not be a response.
    /// Similar to `()` but more verbose.
    pub struct Cancel;

    impl From<Cancel> for Response {
        fn from(_: Cancel) -> Response {
            Response(None)
        }
    }

    impl From<()> for Response {
        fn from(_: ()) -> Response {
            Response(None)
        }
    }

    pub struct Empty;

    impl From<Empty> for Response {
        fn from(_: Empty) -> Response {
            Response(Some(Ok(vec![])))
        }
    }

    pub struct Error<T>(pub T);

    impl<T: Into<ResponsePart>> From<Error<T>> for Response {
        fn from(error: Error<T>) -> Response {
            Response(Some(Err(vec![error.0.into()])))
        }
    }

    impl<T: Into<Response>> From<Option<T>> for Response {
        fn from(option: Option<T>) -> Response {
            match option {
                Some(reponse) => reponse.into(),
                None => Response(None),
            }
        }
    }

    impl<T: Into<Response>, E: Display> From<Result<T, E>> for Response {
        fn from(result: Result<T, E>) -> Response {
            match result {
                Ok(reponse) => reponse.into(),
                Err(err) => Response(Some(Err(vec![Part {
                    kind: Kind::Text,
                    content: err.to_string().as_bytes().into(),
                }]))),
            }
        }
    }

    impl<P: Into<ResponsePart>> From<P> for Response {
        fn from(part: P) -> Response {
            Response(Some(Ok(vec![part.into()])))
        }
    }

    pub struct Body<T>(pub T);

    impl<P1: Into<ResponsePart>, P2: Into<ResponsePart>> From<Body<(P1, P2)>> for Response {
        fn from(Body((p1, p2)): Body<(P1, P2)>) -> Response {
            Response(Some(Ok(vec![p1.into(), p2.into()])))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(_: impl Service) {}

    #[test]
    fn from_cancel() {
        service(|_| async { Cancel });
    }

    #[test]
    fn from_unit() {
        service(|_| async { () });
    }

    #[test]
    fn from_empty() {
        service(|_| async { Empty });
    }

    #[test]
    fn from_str() {
        service(|_| async { "value" });
    }

    #[test]
    fn from_str_attachment() {
        service(|_| async { ("name", "content") });
    }

    #[test]
    fn from_string_attachment() {
        service(|_| async { ("name", String::from("content")) });
    }

    #[test]
    fn from_vec_u8_attachment() {
        service(|_| async { ("name", vec![0x65]) });
    }

    #[test]
    fn from_error() {
        service(|_| async { Error("This is an error") });
    }

    #[test]
    fn from_n_parts() {
        service(|_| async { Body(("value", ("name", vec![0x65]))) });
    }

    #[test]
    fn from_option() {
        async fn handler(req: Request) -> Option<impl Into<Response>> {
            match req.body.len() {
                1 => Some("response"),
                _ => None,
            }
        }

        service(handler);
    }

    #[test]
    fn from_result() {
        async fn handler(req: Request) -> Result<impl Into<Response>, Box<dyn std::error::Error>> {
            match req.body.len() {
                1 => Ok("Correct response"),
                _ => Err("Error response")?,
            }
        }

        service(handler);
    }
}
