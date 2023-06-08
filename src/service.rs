use std::future::Future;

use async_trait::async_trait;

use crate::message::{Message, Part};

#[async_trait]
pub trait Service: Send + Clone + 'static {
    async fn process(self, msg: Message) -> Vec<Part>;
}

#[async_trait]
impl<F, Fut> Service for F
where
    F: FnOnce(Message) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Vec<Part>> + Send,
{
    async fn process(self, msg: Message) -> Vec<Part> {
        (self)(msg).await
    }
}
