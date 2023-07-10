![Crates.io](https://img.shields.io/crates/v/mailfred)
![docs.rs](https://img.shields.io/docsrs/mailfred/latest)
![Crates.io](https://img.shields.io/crates/l/mailfred)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/lemunozm/mailfred/ci.yml)
[![](https://img.shields.io/badge/bymeacoffee-donate-yellow)](https://www.buymeacoffee.com/lemunozm)


# Mailfred

Expose services through the email infrastructure.

## Motivation

The simple fact of setting up a custom service to allow users to use it is not easy.
You need to deploy your server, pay for resources, host it through a public address,
and provide a client application that they can use to interact with your service.

This is a lot of work and maintenance.
Sometimes it can be justified, but sometimes the only purpose is to expose a simple service they can use,
and you do not want to deal with all this amount of work.

`mailfred` tries to give a solution to this using the current email infrastructure.
It reads emails from an email account, fetches them, processes them, and reply them back to the remitter.
It does not act as an email server; it acts as a client (using SMTP and IMAP protocols) that connects to an email service provider.
You don't need to set up and deploy a server email.
You don't need to host anything or buy a public domain address to make it accessible
(you can run it from your own home if you want).
And more important, all your users already have your client application in their mobiles and computers:
their own email client applications that they already know how to use.

## How does it works?
`mailfred` is inspired by [`axum`](https://github.com/tokio-rs/axum).
It works as an HTTP server, but instead of connecting through a *TCP transport* on *port 80*,
it is connected through *IMAP and SMTP protocols* on an *email address*.

Each email sent to that email address is fetched and interpreted as if it was an HTTP request.
The request email is routed to the correct service using the *subject*.
Once it is processed by the service, a new email is sent back to the remitter as if it was an HTTP response.

![image](https://github.com/lemunozm/mailfred/assets/15687891/7366bc2c-6d70-45d8-af3d-20f38edc3696)

## Documentation
- [API documentation](https://docs.rs/mailfred/)
- [Architecture diagram](docs/architecture.md)
- [Examples](examples)

## Getting started
Add the following to your `Cargo.toml`:
```toml
mailfred = "0.1"
tokio = { version = "1", features = ["full"] }
```
[`tokio`](https://github.com/tokio-rs/tokio) is required to run the async tasks.

## Example 
```rust,no_run
use mailfred::{
    router::{Router, layers::LowercaseHeader},
    service::{user_error, Request, Response, ResponseResult},
    transports::Gmail,
};
use tokio::sync::Mutex;
use std::sync::Arc;

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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let gmail = Gmail::new("user", "1234");

    let router = Router::default()
        .route("Count", count)
        .route("Echo", echo)
        .layer(LowercaseHeader);

    mailfred::serve(gmail, State::default(), router).await
}
```

## Gmail account configuration
If you want to use a *Gmail* account, you need to set up some things before:
1. Create a new account, do NOT use your normal account. The `mailfred`'s *IMAP* transport removes the messages it reads from the inbox.
2. Enable *IMAP* in the *Gmail* configuration.
3. Enable [Gmail's app passwords](https://support.google.com/accounts/answer/185833?hl=en) for the account.
