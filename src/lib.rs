pub mod connector;
pub mod message;
pub mod transports;

use std::{error::Error, sync::Arc, time::Duration};

use connector::{Connector, Inbound, Outbound};
use message::{Message, Processor, Receiver, Sender, Transport};
use tokio::sync::Mutex;

const MAX_DELAY_CONNECTION_RETRY: Duration = Duration::from_secs(256);

pub async fn serve(
    connector: impl Connector,
    processor: impl Processor,
) -> Result<(), Box<dyn Error>> {
    let (inbound, outbound) = connector.split();

    let mut receiver = ConnectionHandler::connect(inbound).await?;
    let sender = ConnectionHandler::connect(outbound).await?;

    let shared_sender = Arc::new(Mutex::new(sender));

    loop {
        let input = receiver.recv().await;
        let shared_sender = shared_sender.clone();
        let processor = processor.clone();

        tokio::spawn(async move {
            let address = input.address.clone();
            let subject = input.subject.clone();

            log::info!(
                "Process message for '{}' with subject '{}'",
                address,
                subject
            );

            let output = Message {
                address,
                subject,
                body: processor.process(input).await,
            };

            let mut sender = shared_sender.lock().await;
            sender.send(&output).await
        });
    }
}

pub struct ConnectionHandler<T: Transport> {
    transport: T,
    conn: T::Connection,
}

impl<T: Transport> ConnectionHandler<T> {
    pub async fn connect(transport: T) -> Result<Self, T::Error> {
        let handler = Self {
            conn: transport.connect().await?,
            transport,
        };

        log::info!("{}: connected!", T::NAME);

        Ok(handler)
    }

    async fn force_connect(&mut self) {
        let mut waiting = Duration::from_secs(1); //secs
        loop {
            log::trace!("{}: trying to reconnect...", T::NAME);
            match self.transport.connect().await {
                Ok(conn) => {
                    self.conn = conn;
                    log::info!("{}: reconnected!", T::NAME);
                }
                Err(_) => {
                    log::warn!(
                        "{}: reconnection failed, retry in {}",
                        T::NAME,
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
                    log::trace!("{}: message received from '{}'", T::NAME, msg.address);
                    break msg;
                }
                Err(_) => {
                    log::warn!("{}: message could not be received", T::NAME);
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
                    log::trace!("{}: message sent to {}", T::NAME, msg.address);
                    break;
                }
                Err(_) => {
                    log::warn!("{}: message could not be sent to {}", T::NAME, msg.address);
                    self.force_connect().await
                }
            }
        }
    }
}
