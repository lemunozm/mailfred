use std::{error::Error, fmt::Display, future::Future};

use async_trait::async_trait;

use crate::transport::{Kind, Message, Part};

pub type Request = Message;
pub struct Response(pub Option<Vec<Part>>);
pub type ResponseResult<R> = Result<R, Box<dyn Error>>;

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

/// Type to indicate will not be a response.
/// Similar to `()` but more verbose.
struct Cancel;

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

struct Empty;

impl From<Empty> for Response {
    fn from(_: Empty) -> Response {
        Response(Some(vec![]))
    }
}

impl<'a> From<&'a str> for Response {
    fn from(value: &'a str) -> Response {
        Response(Some(vec![Part {
            kind: Kind::Text,
            content: value.as_bytes().into(),
        }]))
    }
}

impl From<String> for Response {
    fn from(value: String) -> Response {
        Response(Some(vec![Part {
            kind: Kind::Text,
            content: value.as_bytes().into(),
        }]))
    }
}

impl<'a, N: AsRef<str>> From<(N, &'a str)> for Response {
    fn from((name, content): (N, &'a str)) -> Response {
        Response(Some(vec![Part {
            kind: Kind::Attachment(name.as_ref().into()),
            content: content.as_bytes().into(),
        }]))
    }
}

impl<N: AsRef<str>> From<(N, String)> for Response {
    fn from((name, content): (N, String)) -> Response {
        Response(Some(vec![Part {
            kind: Kind::Attachment(name.as_ref().into()),
            content: content.as_bytes().into(),
        }]))
    }
}

impl<N: AsRef<str>> From<(N, Vec<u8>)> for Response {
    fn from((name, content): (N, Vec<u8>)) -> Response {
        Response(Some(vec![Part {
            kind: Kind::Attachment(name.as_ref().into()),
            content: content.into(),
        }]))
    }
}

impl<T: Into<Response>, E: Display> From<Result<T, E>> for Response {
    fn from(result: Result<T, E>) -> Response {
        match result {
            Ok(reponse) => reponse.into(),
            Err(err) => err.to_string().into(),
        }
    }
}

/*
impl<T1: Into<Response>, T2: Into<Response>> From<(T1, T2)> for Response {
    fn from((t1, t2): (T1, T2) -> Response {
        t1.0?
        Response(vec![t1.])
    }
}
*/

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
    fn from_result() {
        async fn handler(req: Request) -> ResponseResult<impl Into<Response>> {
            let value = match req.body.len() {
                1 => "Correct response",
                _ => Err("error")?,
            };

            Ok(value)
        }

        service(handler);
    }
}
