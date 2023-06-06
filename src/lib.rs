pub mod connector;
pub mod message;
pub mod transports;

use std::{error::Error, sync::Arc};

use connector::Connector;
use message::{Message, Processor};
use tokio::sync::Mutex;

pub async fn serve<C, P>(connector: C, processor: P) -> Result<(), Box<dyn Error>>
where
    C: Connector,
    P: Processor,
{
    let (mut receiver, sender) = connector.connect_full_duplex().await?;
    let shared_sender = Arc::new(Mutex::new(sender));

    loop {
        let message = receiver.recv().await;
        let shared_sender = shared_sender.clone();
        let mut processor = processor.clone();

        tokio::spawn(async move {
            let body = processor.process(&message).await;
            let msg = Message { body, ..message };
            let mut sender = shared_sender.lock().await;
            sender.send(&msg).await
        });
    }
}

/*
#[derive(Clone)]
struct EchoApp;

#[async_trait]
impl Processor for EchoApp {
    async fn process(&mut self, msg: &Message) -> Vec<Part> {
        msg.body.clone()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn server() {
    mailfred::serve(default_connector(), EchoApp).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn run_and_stop() {
    let handle = tokio::spawn(async move {
        mailfred::serve(default_connector(), EchoApp).await.unwrap();
    });

    tokio::time::sleep(Duration::from_secs(5)).await;

    handle.abort();
}
*/
