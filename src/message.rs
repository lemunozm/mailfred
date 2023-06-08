use std::{error::Error, future::Future};

use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Text,
    Html,
    Attachment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub kind: Kind,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub address: String,
    pub header: String,
    pub body: Vec<Part>,
}

#[async_trait]
pub trait Transport: Sync + Send {
    const NAME: &'static str;

    type Connection: Send;
    type Error: Send + Error + 'static;

    async fn connect(&self) -> Result<Self::Connection, Self::Error>;
}

#[async_trait]
pub trait Sender: Sized + Send {
    type Error: Send + Error + 'static;

    async fn send(&mut self, msg: &Message) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait Receiver: Sized + Send {
    type Error: Send + Error + 'static;

    async fn recv(&mut self) -> Result<Message, Self::Error>;
}

#[async_trait]
pub trait Processor: Send + Clone + 'static {
    async fn process(self, msg: Message) -> Vec<Part>;
}

#[async_trait]
impl<F, Fut> Processor for F
where
    F: FnOnce(Message) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Vec<Part>> + Send,
{
    async fn process(self, msg: Message) -> Vec<Part> {
        (self)(msg).await
    }
}
