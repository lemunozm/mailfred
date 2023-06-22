use std::sync::Arc;

use mailfred::{
    router::Router,
    service::{user_error, Request, Response, ResponseResult},
    transports::Gmail,
};
use tokio::sync::Mutex;

#[derive(Default)]
struct MyState {
    counter: u32,
}

type State = Arc<Mutex<MyState>>;

async fn count(_: Request, state: State) -> ResponseResult {
    let mut state = state.lock().await;
    state.counter += 1;

    Response::ok("Counter stats", format!("Value: {}", state.counter))
}

async fn echo(req: Request, _: State) -> ResponseResult {
    Response::ok(req.header, req.body)
}

async fn sum_csv(req: Request, _: State) -> ResponseResult {
    let attachment = req
        .attachment_iter()
        .next()
        .ok_or("Expected a csv attachment")
        .map_err(user_error)?;

    let mut csv = csv::Reader::from_reader(attachment.content.as_slice());

    let mut total = 0;
    for record in csv.records() {
        for elem in record.map_err(user_error)?.iter() {
            total = elem.parse().map_err(user_error)?;
        }
    }

    Response::ok(req.header, format!("Total: {}", total))
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "logger")]
    mailfred::util::logger::configure(log::LevelFilter::Trace);

    let gmail = Gmail::new("user", "1234");

    let router = Router::default()
        .route("Count", count)
        .route("Echo", echo)
        .route("Sum", sum_csv);

    mailfred::serve(gmail, State::default(), router).await
}
