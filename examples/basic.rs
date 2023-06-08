use mailfred::{
    self, logger,
    message::{Message, Part},
    transports::{Imap, Smtp},
};

async fn echo(msg: Message) -> Vec<Part> {
    msg.body.clone()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::configure(log::LevelFilter::Trace);

    let imap = Imap {
        domain: "imap.gmail.com".into(),
        port: 993,
        user: "user@gmail.com".into(),
        password: "1234".into(),
        folder: "inbox".into(),
    };

    let smtp = Smtp {
        domain: "smtp.gmail.com".into(),
        port: 587,
        user: "user@gmail.com".into(),
        password: "1234".into(),
    };

    mailfred::serve((imap, smtp), echo).await
}
