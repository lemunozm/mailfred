use std::{error::Error, time::Duration};

use async_trait::async_trait;

use crate::message::{Message, Receiver, Sender, Transport};

const MAX_DELAY_CONNECTION_RETRY: Duration = Duration::from_secs(256);

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

pub struct ConnectionHandler<T: Transport> {
    transport: T,
    conn: T::Connection,
}

impl<T: Transport> ConnectionHandler<T> {
    pub async fn connect(transport: T) -> Result<Self, T::Error> {
        Ok(Self {
            conn: transport.connect().await?,
            transport,
        })
    }

    async fn force_connect(&mut self) {
        let mut waiting = Duration::from_secs(1); //secs
        loop {
            match self.transport.connect().await {
                Ok(conn) => self.conn = conn,
                Err(_) => {
                    tokio::time::sleep(waiting).await;
                    waiting = (waiting * 2).max(MAX_DELAY_CONNECTION_RETRY);
                }
            }
        }
    }
}

impl<T: Inbound> ConnectionHandler<T> {
    pub async fn recv(&mut self) -> Message {
        loop {
            match self.conn.recv().await {
                Ok(msg) => break msg,
                Err(_) => self.force_connect().await,
            }
        }
    }
}

impl<T: Outbound> ConnectionHandler<T> {
    pub async fn send(&mut self, msg: &Message) {
        loop {
            match self.conn.send(&msg).await {
                Ok(_) => break,
                Err(_) => self.force_connect().await,
            }
        }
    }
}

pub type FullDuplex<C> = (
    ConnectionHandler<<C as Connector>::Inbound>,
    ConnectionHandler<<C as Connector>::Outbound>,
);

#[async_trait]
pub trait Connector: Sized + Sync + Send {
    type Inbound: Inbound;
    type Outbound: Outbound;

    fn split(self) -> (Self::Inbound, Self::Outbound);

    async fn connect_full_duplex(self) -> Result<FullDuplex<Self>, Box<dyn Error>> {
        let (inbound, outbound) = self.split();

        let receiver = ConnectionHandler::connect(inbound).await?;
        let sender = ConnectionHandler::connect(outbound).await?;

        Ok((receiver, sender))
    }
}

impl<I: Inbound, O: Outbound> Connector for (I, O) {
    type Inbound = I;
    type Outbound = O;

    fn split(self) -> (Self::Inbound, Self::Outbound) {
        self
    }
}
