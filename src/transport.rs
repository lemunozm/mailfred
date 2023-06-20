use std::error::Error;

use async_trait::async_trait;

use crate::message::Message;

#[async_trait]
pub trait Transport: Sync + Send {
    const NAME: &'static str;

    type Connection: Send;
    type Error: Error + Send + Sync + 'static;

    async fn connect(&self) -> Result<Self::Connection, Self::Error>;
}

#[async_trait]
pub trait Sender: Sized + Send {
    type Error: Error + Send + Sync + 'static;

    async fn send(&mut self, msg: &Message) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait Receiver: Sized + Send {
    type Error: Error + Send + Sync + 'static;

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
