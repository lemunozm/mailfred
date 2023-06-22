use mailfred::{
    service::{Request, Response, ResponseResult},
    transports::Gmail,
};

async fn echo(req: Request, _state: ()) -> ResponseResult {
    Response::ok(req.header, req.body)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "logger")]
    mailfred::util::logger::configure(log::LevelFilter::Trace);

    let gmail = Gmail {
        username: "user".into(),
        password: "1234".into(),
    };

    mailfred::serve(gmail, (), echo).await
}
