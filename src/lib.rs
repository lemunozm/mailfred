mod connection_handler;
pub mod message;
pub mod router;
pub mod service;
pub mod transport;
pub mod transports;
pub mod util {
    #[cfg(feature = "logger")]
    pub mod logger;
}

use std::sync::Arc;

use connection_handler::PerpetualConnection;
use message::Message;
use service::{ErrorResponse, Service};
use tokio::sync::Mutex;
use transport::{Connector, Inbound};

pub async fn serve<S: Clone + Send + 'static>(
    connector: impl Connector,
    shared_state: S,
    service: impl Service<S>,
) -> Result<(), anyhow::Error> {
    let (inbound, outbound) = connector.split();

    let mut receiver = PerpetualConnection::connect(inbound, "").await?;
    let sender = PerpetualConnection::connect(outbound, "").await?;

    let shared_sender = Arc::new(Mutex::new(sender));
    let shared_service = Arc::new(service);

    loop {
        let input = receiver.recv().await;
        let sender = shared_sender.clone();
        let service = shared_service.clone();
        let state = shared_state.clone();

        tokio::spawn(async move {
            let address = input.address.clone();
            let header = input.header.clone();

            log::info!("Process message for '{}' with header '{}'", address, header);

            let response = match service.call(input, state).await {
                Ok(response) => response?,
                Err(ErrorResponse::User(response)) => response,
                Err(ErrorResponse::System(response)) => {
                    log::error!("System error: {}", response.body.to_string());
                    response
                }
            };

            let output = Message {
                address,
                header: response.header,
                body: response.body.0,
            };

            let mut sender = sender.lock().await;
            sender.send(&output).await;

            Some(())
        });
    }
}

pub async fn init_consumer_task<T: Inbound>(imap: T, log_suffix: &str) -> Result<(), T::Error> {
    let log_suffix = format!("{}-consumer", log_suffix);
    let mut consumer = PerpetualConnection::connect(imap, &log_suffix).await?;

    tokio::spawn(async move {
        loop {
            let _ = consumer.recv().await;
        }
    });

    Ok(())
}
