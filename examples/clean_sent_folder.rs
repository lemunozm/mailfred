use mailfred::{
    self,
    connector::Connector,
    logger,
    message::{Message, Part},
    transports::{Gmail, Imap},
};

async fn echo(msg: Message) -> Option<Vec<Part>> {
    Some(msg.body.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::configure(log::LevelFilter::Trace);

    let (imap, smtp) = Gmail {
        username: "user".into(),
        password: "1234".into(),
    }
    .split();

    // Modify the default imap folder to use the sent folder instead.
    // Each email server can have each own name for this.
    let clean_sent_imap = Imap {
        folder: "[Gmail]/Sent".into(),
        ..imap.clone()
    };

    // Create a imap connection to consume all messages from that folder
    mailfred::init_consumer(clean_sent_imap, "sent").await?;

    mailfred::serve((imap, smtp), echo).await
}
