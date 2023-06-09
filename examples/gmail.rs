use mailfred::{
    self, logger,
    transport::{Message, Part},
    transports::Gmail,
};

async fn echo(msg: Message) -> Option<Vec<Part>> {
    Some(msg.body.clone())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::configure(log::LevelFilter::Trace);

    let gmail = Gmail {
        username: "user".into(),
        password: "1234".into(),
    };

    mailfred::serve(gmail, echo).await
}
