mod connection_handler;
#[cfg(feature = "logger")]
pub mod logger;
pub mod router;
pub mod service;
pub mod transport;
pub mod transports;

use std::{error::Error, sync::Arc};

use connection_handler::ConnectionHandler;
use service::Service;
use tokio::sync::Mutex;
use transport::{Connector, Inbound, Message};

pub async fn serve<S: Clone + Send + 'static>(
    connector: impl Connector,
    state: S,
    service: impl Service<S>,
) -> Result<(), Box<dyn Error>> {
    let (inbound, outbound) = connector.split();

    let mut receiver = ConnectionHandler::connect(inbound, "").await?;
    let sender = ConnectionHandler::connect(outbound, "").await?;

    let shared_sender = Arc::new(Mutex::new(sender));
    let shared_service = Arc::new(service);

    loop {
        let input = receiver.recv().await;
        let sender = shared_sender.clone();
        let service = shared_service.clone();
        let state = state.clone();

        tokio::spawn(async move {
            let address = input.address.clone();
            let header = input.header.clone();

            log::info!("Process message for '{}' with header '{}'", address, header);

            let output = Message {
                address,
                header,
                body: match service.call(input, state).await.into().0? {
                    Ok(body) => body,
                    Err(body) => body,
                },
            };

            let mut sender = sender.lock().await;
            sender.send(&output).await;

            Some(())
        });
    }
}

pub async fn init_consumer_task<T: Inbound>(
    imap: T,
    log_suffix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_suffix = format!("{}-consumer", log_suffix);
    let mut consumer = ConnectionHandler::connect(imap, &log_suffix).await?;

    tokio::spawn(async move {
        loop {
            let _ = consumer.recv().await;
        }
    });

    Ok(())
}
