use std::error::Error;

use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub headers: Vec<String>,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub address: String,
    pub subject: String,
    pub body: Vec<Part>,
}

#[async_trait]
pub trait Transport: Sync + Send {
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
    async fn process(&mut self, msg: &Message) -> Vec<Part>;
}