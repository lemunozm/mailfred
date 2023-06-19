use std::time::Duration;

use crate::transport::{Inbound, Message, Outbound, Receiver, Sender, Transport};

const MAX_DELAY_CONNECTION_RETRY: Duration = Duration::from_secs(256);

pub struct ConnectionHandler<T: Transport> {
    transport: T,
    conn: T::Connection,
    log_name: String,
}

impl<T: Transport> ConnectionHandler<T> {
    pub async fn connect(transport: T, log_suffix: &str) -> Result<Self, T::Error> {
        let log_name = format!("{}-{}", T::NAME, log_suffix);
        Ok(Self {
            conn: match transport.connect().await {
                Ok(conn) => {
                    log::info!("{}: connected!", log_name);
                    conn
                }
                Err(err) => {
                    log::error!("{}: can not connect", log_name);
                    Err(err)?
                }
            },
            transport,
            log_name,
        })
    }

    async fn force_connect(&mut self) {
        let mut waiting = Duration::from_secs(1); //secs
        loop {
            log::trace!("{}: trying to reconnect...", self.log_name);
            match self.transport.connect().await {
                Ok(conn) => {
                    self.conn = conn;
                    log::info!("{}: reconnected!", self.log_name);
                    break;
                }
                Err(_) => {
                    log::warn!(
                        "{}: reconnection failed, retry in {}",
                        self.log_name,
                        waiting.as_secs()
                    );
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
                Ok(msg) => {
                    log::trace!("{}: message received from '{}'", self.log_name, msg.address);
                    break msg;
                }
                Err(_) => {
                    log::warn!("{}: message could not be received", self.log_name);
                    self.force_connect().await
                }
            }
        }
    }
}

impl<T: Outbound> ConnectionHandler<T> {
    pub async fn send(&mut self, msg: &Message) {
        loop {
            match self.conn.send(&msg).await {
                Ok(_) => {
                    log::trace!("{}: message sent to {}", self.log_name, msg.address);
                    break;
                }
                Err(_) => {
                    log::warn!(
                        "{}: message could not be sent to {}",
                        self.log_name,
                        msg.address
                    );
                    self.force_connect().await
                }
            }
        }
    }
}
