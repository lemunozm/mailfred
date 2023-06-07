pub mod connector;
pub mod message;
pub mod transports;

use std::{error::Error, sync::Arc};

use connector::Connector;
use message::{Message, Processor};
use tokio::sync::Mutex;

pub async fn serve(
    connector: impl Connector,
    processor: impl Processor,
) -> Result<(), Box<dyn Error>> {
    let (mut receiver, sender) = connector.connect_full_duplex().await?;
    let shared_sender = Arc::new(Mutex::new(sender));

    loop {
        let input = receiver.recv().await;
        let shared_sender = shared_sender.clone();
        let processor = processor.clone();

        tokio::spawn(async move {
            let output = Message {
                address: input.address.clone(),
                subject: input.subject.clone(),
                body: processor.process(input).await,
            };

            let mut sender = shared_sender.lock().await;
            sender.send(&output).await
        });
    }
}
