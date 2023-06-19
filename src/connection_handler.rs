use std::time::Duration;

use crate::transport::{Inbound, Message, Outbound, Receiver, Sender, Transport};

const MAX_RECONNETION_ATTEMPS: u32 = 10;

pub struct ConnectionHandler<T: Transport> {
    transport: T,
    conn: T::Connection,
    log_name: String,
}

impl<T: Transport> ConnectionHandler<T> {
    pub async fn connect(transport: T, log_suffix: &str) -> Result<Self, T::Error> {
        let log_name = format!(
            "{}{}{}",
            T::NAME,
            if log_suffix.is_empty() { "" } else { "-" },
            log_suffix
        );

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
        let mut attempts: u32 = 0;
        loop {
            log::trace!("{}: trying to reconnect...", self.log_name);
            match self.transport.connect().await {
                Ok(conn) => {
                    self.conn = conn;
                    log::info!("{}: reconnected!", self.log_name);
                    break;
                }
                Err(_) => {
                    let waiting_secs = 2u64.pow(attempts);

                    log::debug!(
                        "{}: reconnection failed, retry in {}",
                        self.log_name,
                        waiting_secs
                    );

                    tokio::time::sleep(Duration::from_secs(waiting_secs)).await;

                    if attempts == MAX_RECONNETION_ATTEMPS - 1 {
                        log::warn!(
                            "{}: Connection issue, more than {}",
                            self.log_name,
                            attempts
                        );
                    }

                    attempts = (attempts + 1).max(MAX_RECONNETION_ATTEMPS);
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
                    log::debug!("{}: receiver connection lost", self.log_name);
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
                    log::debug!(
                        "{}: sender connection lost. Trying to send to {}",
                        self.log_name,
                        msg.address
                    );
                    self.force_connect().await
                }
            }
        }
    }
}
