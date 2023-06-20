use mailfred::{
    self,
    service::{
        response::{Response, ResponseResult},
        Request,
    },
    transports::Gmail,
    util::logger,
};

async fn echo(req: Request, _state: ()) -> ResponseResult {
    Response::ok(req.header, req.body)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    logger::configure(log::LevelFilter::Trace);

    let gmail = Gmail {
        username: "user".into(),
        password: "1234".into(),
    };

    mailfred::serve(gmail, (), echo).await
}
