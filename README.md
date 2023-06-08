# Mailfred

Connect by SMTP and IMAP protocols and process income emails as if it were requests in a HTTP server,
giving a corresponding email response.

:construction: Under construction :construction:

## Notes
[link](https://support.google.com/accounts/answer/185833)

## TODO
- UML
- Testing
- Documentation
- Return nothing from process

## API ideas
```rust
struct MyState;

let state = Arc<MyState>

async fn greet() -> impl IntoResponse {
    "hello, I'm mailfred"
}

let sub_router = Router::builder()
    .service("subcmd", SubCmd)
    .layer(Sign::with("body-part"))

let router = Router::builder()
    .service("echo", Echo)
    .service("with-state", WithState(state))
    .service("greet", greet)
    .service_whitelist("echo", Echo, ["admin@domain.com"])
    .service("cmd", sub_router)
    .layer(LowercaseRoute)
    .layer(Sign::with("body-part"))

struct Request {
    origin: EmailAddress,
    service: String
    args: Vec<String>,
    parts: Vec<Part>,
}

struct Response {
    parts: Vec<Part>,
}

//TODO: make extractors
```
