use std::{fmt::Display, future::Future};

use async_trait::async_trait;
pub use response::{Body, Cancel, Empty, Error, Response, ResponseResult};
pub use response_part::{Html, ResponsePart};

use crate::transport::{Kind, Message, Part};

pub type Request = Message;

#[async_trait]
pub trait Service<State>: Send + Sync + 'static {
    type Response: Into<Response>;

    async fn call(&self, req: Request, state: State) -> Self::Response;
}

#[async_trait]
impl<State, F, Fut, Res> Service<State> for F
where
    F: Fn(Request, State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    Res: Into<Response>,
    State: Send + 'static,
{
    type Response = Res;

    async fn call(&self, req: Request, state: State) -> Self::Response {
        (self)(req, state).await
    }
}

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

pub mod response {
    use super::*;

    pub struct Response(pub Option<Result<Vec<Part>, Vec<Part>>>);

    pub type ResponseResult = Result<Response, Box<dyn std::error::Error>>;

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
        fn from(error: Error<T>) -> Self {
            Response(Some(Err(vec![error.0.into()])))
        }
    }

    impl<T: Into<Response>> From<Option<T>> for Response {
        fn from(option: Option<T>) -> Self {
            match option {
                Some(reponse) => reponse.into(),
                None => Response(None),
            }
        }
    }

    impl<T: Into<Response>, E: Display> From<Result<T, E>> for Response {
        fn from(result: Result<T, E>) -> Self {
            match result {
                Ok(reponse) => reponse.into(),
                Err(err) => Response(Some(Err(vec![Part {
                    kind: Kind::Text,
                    content: err.to_string().as_bytes().into(),
                }]))),
            }
        }
    }

    impl From<Vec<Part>> for Response {
        fn from(body: Vec<Part>) -> Self {
            Response(Some(Ok(body)))
        }
    }

    impl<P: Into<ResponsePart>> From<P> for Response {
        fn from(part: P) -> Self {
            Response(Some(Ok(vec![part.into()])))
        }
    }

    pub struct Body<T>(pub T);

    impl<P1: Into<ResponsePart>, P2: Into<ResponsePart>> From<Body<(P1, P2)>> for Response {
        fn from(Body((p1, p2)): Body<(P1, P2)>) -> Self {
            Response(Some(Ok(vec![p1.into(), p2.into()])))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(_: impl Service<()>) {}

    #[test]
    fn from_cancel() {
        process(|_, _| async { Cancel });
    }

    #[test]
    fn from_unit() {
        process(|_, _| async {});
    }

    #[test]
    fn from_empty() {
        process(|_, _| async { Empty });
    }

    #[test]
    fn from_str() {
        process(|_, _| async { "value" });
    }

    #[test]
    fn from_str_attachment() {
        process(|_, _| async { ("name", "content") });
    }

    #[test]
    fn from_string_attachment() {
        process(|_, _| async { ("name", String::from("content")) });
    }

    #[test]
    fn from_vec_u8_attachment() {
        process(|_, _| async { ("name", vec![0x65]) });
    }

    #[test]
    fn from_error() {
        process(|_, _| async { Error("This is an error") });
    }

    #[test]
    fn from_vec_parts() {
        process(|_, _| async { vec!["value".into(), ("name", vec![0x65]).into()] });
    }

    #[test]
    fn from_body() {
        process(|_, _| async { Body(("value", ("name", vec![0x65]))) });
    }

    #[test]
    fn from_option() {
        async fn service(req: Request, _: ()) -> Option<impl Into<Response>> {
            match req.body.len() {
                1 => Some("response"),
                _ => None,
            }
        }

        process(service);
    }

    #[test]
    fn from_result() {
        async fn service(
            req: Request,
            _: (),
        ) -> Result<impl Into<Response>, Box<dyn std::error::Error>> {
            match req.body.len() {
                1 => Ok("Correct response"),
                _ => Err("Error response")?,
            }
        }

        process(service);
    }
}
