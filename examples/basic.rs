use std::sync::Arc;

use mailfred::{
    self,
    service::{Request, Response, ResponseResult},
    transports::{Imap, Smtp},
};
use tokio::sync::Mutex;

#[derive(Default)]
struct MyState {
    counter: u32,
}

type State = Arc<Mutex<MyState>>;

async fn count(req: Request, state: State) -> ResponseResult {
    let mut state = state.lock().await;
    state.counter += 1;

    Response::ok(req.header, format!("Counter: {}", state.counter))
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(feature = "logger")]
    mailfred::util::logger::configure(log::LevelFilter::Trace);

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

    mailfred::serve((imap, smtp), State::default(), count).await
}
