use std::error::Error;

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

/*
impl Message {
    pub fn part(kind Kind) -> Part {

    }

    pub fn multipart<K1, K2> -> (Option<Part>, Option<Part>) {

    }
}
*/

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

pub trait Inbound: Transport<Connection = Self::InboundQueue> + 'static {
    type InboundQueue: Receiver;
}
impl<T: Transport<Connection = C> + 'static, C: Receiver> Inbound for T {
    type InboundQueue = C;
}

pub trait Outbound: Transport<Connection = Self::OutboundQueue> + 'static {
    type OutboundQueue: Sender;
}
impl<T: Transport<Connection = C> + 'static, C: Sender> Outbound for T {
    type OutboundQueue = C;
}

#[async_trait]
pub trait Connector: Sized + Sync + Send {
    type Inbound: Inbound;
    type Outbound: Outbound;

    fn split(self) -> (Self::Inbound, Self::Outbound);
}

impl<I: Inbound, O: Outbound> Connector for (I, O) {
    type Inbound = I;
    type Outbound = O;

    fn split(self) -> (Self::Inbound, Self::Outbound) {
        self
    }
}
