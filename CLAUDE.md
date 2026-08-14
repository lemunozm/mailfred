# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`mailfred` is a Rust library (published crate) that exposes services over the email infrastructure.
It is not an email server: it is an IMAP/SMTP *client* that fetches emails from an account, treats each
one as a request, routes it by *subject*, and replies to the sender with the response. The API is
deliberately modeled on [`axum`](https://github.com/tokio-rs/axum) — `serve(connector, state, service)`,
`Router::route(filter, handler)`, `.layer(...)`.

## Commands

```bash
cargo build                       # default features: smtp + imap
cargo build --all-features        # adds the `logger` feature
cargo fmt --all -- --check        # CI gate (rustfmt.toml uses nightly-only options; run with nightly if they are ignored)
cargo clippy -- -D warnings       # CI gate
cargo test                        # includes doctests, which compile the README example
cargo test --test integration_transports roundtrip_sync   # a single integration test
cargo run --example router        # examples need imap+smtp (the default features)
```

Feature-permutation checking uses `cargo-all-features`; `full` is denylisted in `Cargo.toml` since it is
only an alias. When adding a feature-gated module, keep the `#[cfg(feature = ...)]` re-exports in
`src/transports/mod.rs` consistent.

### Integration tests hit a real Gmail account

`tests/integration_transports.rs` requires `MAILFRED_TEST_USER` and `MAILFRED_TEST_PASSWORD` and talks to
live `imap.gmail.com`/`smtp.gmail.com`. The tests are `#[serial_test::serial]` because each one calls
`clear_folder("inbox")`, which **permanently deletes every message in that inbox**. Never point these at a
real mailbox. Without the env vars the tests panic rather than skip.

The README's Rust block is compiled as a doctest via `doc_comment::doctest!` in `src/lib.rs`, so changing
the README example can break `cargo test`.

## Architecture

Data flows: **Inbound transport → `PerpetualConnection` → `serve` loop → `Service` → `PerpetualConnection` → Outbound transport.**

- `src/message.rs` — the single wire type. A `Message` is `{ address, header, body: Vec<Part> }`, where each
  `Part` is `Kind::Text | Html | Attachment(name)` plus raw bytes. `header` is the email subject and is what
  routing keys off; `address` is the remitter on the way in and the recipient on the way out.

- `src/transport.rs` — the transport abstraction, layered so a type only has to implement `Transport` +
  (`Sender` or `Receiver`). Blanket impls then derive `Outbound`/`Inbound` from that, and any `(I, O)` tuple
  is automatically a `Connector`. Adding a transport means implementing `Transport` (with an associated
  `Connection`) and `Sender`/`Receiver` on the connection — nothing else.

- `src/transports/` — concrete transports, each behind its own feature. `Imap` is receive-only, `Smtp` is
  send-only, and `Gmail` is a `Connector` that splits into a preconfigured pair. Note `Imap` **deletes**
  messages it reads (`\Deleted` + `expunge`) — that extraction behavior is intentional and is what makes
  `spawn_consumer` usable to keep a folder clean. The IMAP side runs a blocking `imap` crate session on
  `spawn_blocking` and bridges to async over an mpsc channel; a `Notify` handshake ensures a message is only
  flagged deleted once the async side is actually ready to receive it (no message is lost on shutdown).

  The listener loop has three invariants that are easy to break. **Always address messages by UID**
  (`uid_fetch`/`uid_store`), never by sequence number — sequence numbers are renumbered by any client's
  expunge, so a stale one deletes the wrong message. **Every message must eventually leave the pending
  set**, either deleted or marked with the `mailfred-unprocessable` keyword; if one can be listed forever,
  the loop never reaches its `IDLE` branch and busy-loops over the whole folder. And **`IDLE` is refreshed
  on a short `timeout()` with keepalive left on**, because that timeout doubles as the socket read timeout —
  it is the only thing that detects a connection killed by a suspend or a dropped NAT entry.

- `src/connection.rs` — `PerpetualConnection<T>` wraps a transport so `recv`/`send` have no error type: any
  failure triggers reconnection (with backoff) and retry. This is why `serve` has no error handling in the
  loop body; errors surface only at initial connect.

- `src/lib.rs` — `serve` splits the connector, wraps both halves in `PerpetualConnection`, then loops
  receiving and `tokio::spawn`s per message. Handlers run concurrently; the *sender* is behind an
  `Arc<Mutex<_>>` and is therefore serialized. `serve` never returns once connected — wrap it in a
  `tokio::spawn` if it needs to be cancellable.

- `src/service/` — `Service<State>` is `async fn call(Request, State) -> ResponseResult`, with a blanket impl
  for `Fn(Request, State) -> Future`, so plain async fns are handlers. `ResponseResult` is
  `Result<Option<Response>, ErrorResponse>`: `Ok(None)` means *send no reply* (also what an unmatched route
  yields). `ErrorResponse::User` is echoed back to the sender as-is; `ErrorResponse::System` is logged as an
  error and then also sent. The ergonomics come from `From` impls in `response.rs` — `&str`, `String`, `Html`,
  `(name, content)` tuples for attachments, `Parts((a, b))` for multiple — so extend those impls rather than
  adding new constructors.

- `src/router/` — `Router<State>` is itself a `Service`, so routers nest. `Filter` decides whether a route
  matches the header (`&'static str` for exact match, `Any`, `StartWith`); the **first** matching route wins.
  `Layer` maps request and/or response for *every* message; layers run in insertion order for requests and
  in the same order again for responses.

`docs/architecture.md` holds a PlantUML class diagram of these relationships — update it when trait
relationships change.

## Conventions

- `rustfmt.toml` enables `imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"` and
  `wrap_comments` — these are nightly-only rustfmt options, so stable `cargo fmt` silently ignores them.
- Public items carry doc comments; the crate's docs are the README plus rustdoc.
- New router filters/layers and new transports are the intended extension points (the README explicitly
  invites PRs for them) — put them in `src/router/filters.rs`, `src/router/layers.rs`, or `src/transports/`.
