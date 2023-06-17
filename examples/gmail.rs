use mailfred::{
    self, logger,
    service::{Request, Response},
    transports::Gmail,
};

async fn echo(req: Request, _state: ()) -> impl Into<Response> {
    req.body
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::configure(log::LevelFilter::Trace);

    let gmail = Gmail {
        username: "user".into(),
        password: "1234".into(),
    };

    mailfred::serve(gmail, (), echo).await
}
