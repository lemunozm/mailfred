pub mod response;
use std::future::Future;

use async_trait::async_trait;
pub use response::*;

use crate::message::Message;

pub type Request = Message;

#[async_trait]
pub trait Service<State>: Send + Sync + 'static {
    async fn call(&self, req: Request, state: State) -> ResponseResult;
}

#[async_trait]
impl<State, F, Fut> Service<State> for F
where
    F: Fn(Request, State) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ResponseResult> + Send,
    State: Send + 'static,
{
    async fn call(&self, req: Request, state: State) -> ResponseResult {
        (self)(req, state).await
    }
}
