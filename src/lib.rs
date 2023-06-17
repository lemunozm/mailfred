mod connection_handler;
#[cfg(feature = "logger")]
pub mod logger;
pub mod service;
pub mod transport;
pub mod transports;

use std::{error::Error, sync::Arc};

use connection_handler::ConnectionHandler;
use service::Service;
use tokio::sync::Mutex;
use transport::{Connector, Inbound, Message};

pub async fn serve(connector: impl Connector, service: impl Service) -> Result<(), Box<dyn Error>> {
    let (inbound, outbound) = connector.split();

    let mut receiver = ConnectionHandler::connect(inbound, "main").await?;
    let sender = ConnectionHandler::connect(outbound, "main").await?;

    let shared_sender = Arc::new(Mutex::new(sender));

    loop {
        let input = receiver.recv().await;
        let shared_sender = shared_sender.clone();
        let service = service.clone();

        tokio::spawn(async move {
            let address = input.address.clone();
            let header = input.header.clone();

            log::info!("Process message for '{}' with header '{}'", address, header);

            let output = Message {
                address,
                header,
                body: match service.call(input).await.into().0? {
                    Ok(body) => body,
                    Err(body) => body,
                },
            };

            let mut sender = shared_sender.lock().await;
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
