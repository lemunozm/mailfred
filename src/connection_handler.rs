use std::time::{Duration, Instant};

use crate::{
    message::Message,
    transport::{Inbound, Outbound, Receiver, Sender, Transport},
};

const MAX_RECONN_DELAY: Duration = Duration::from_secs(60);
const LOG_AFTER: Duration = Duration::from_secs(60);

pub struct PerpetualConnection<T: Transport> {
    transport: T,
    conn: T::Connection,
    log_name: String,
}

impl<T: Transport> PerpetualConnection<T> {
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
        let mut warned = false;
        let initial = Instant::now();

        loop {
            match self.transport.connect().await {
                Ok(conn) => {
                    self.conn = conn;

                    let inactivity = Instant::now() - initial;
                    if inactivity > LOG_AFTER {
                        log::info!(
                            "{}: reconnected after an inactivity period of {}:{:02}:{:02}",
                            self.log_name,
                            inactivity.as_secs() / 3600,
                            (inactivity.as_secs() / 60) % 60,
                            inactivity.as_secs() % 60
                        );
                    }

                    break;
                }
                Err(_) => {
                    if Instant::now() - initial > LOG_AFTER && !warned {
                        warned = true;
                        log::warn!(
                            "{}: disconnected for more than {} seconds",
                            LOG_AFTER.as_secs(),
                            self.log_name
                        );
                    }

                    let delay = Duration::from_secs(2u64.pow(attempts)).max(MAX_RECONN_DELAY);
                    attempts += 1;

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

impl<T: Inbound> PerpetualConnection<T> {
    pub async fn recv(&mut self) -> Message {
        loop {
            match self.conn.recv().await {
                Ok(msg) => {
                    log::debug!("{}: message received from '{}'", self.log_name, msg.address);
                    break msg;
                }
                Err(_) => {
                    log::trace!("{}: receiver connection lost", self.log_name);
                    self.force_connect().await
                }
            }
        }
    }
}

impl<T: Outbound> PerpetualConnection<T> {
    pub async fn send(&mut self, msg: &Message) {
        loop {
            match self.conn.send(&msg).await {
                Ok(_) => {
                    log::debug!("{}: message sent to {}", self.log_name, msg.address);
                    break;
                }
                Err(_) => {
                    log::trace!(
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
