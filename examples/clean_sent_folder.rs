use mailfred::{
    service::{Request, Response, ResponseResult},
    transport::Connector,
    transports::{Gmail, Imap},
};

async fn echo(req: Request, _state: ()) -> ResponseResult {
    Response::ok(req.header, req.body)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "logger")]
    mailfred::util::logger::configure(log::LevelFilter::Trace);

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
    mailfred::spawn_consumer(clean_sent_imap, "sent").await?;

    mailfred::serve((imap, smtp), (), echo).await
}
