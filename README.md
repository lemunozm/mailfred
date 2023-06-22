# Mailfred

Expose services through the email infrastructure.

## Motivation

The simple fact of setting up a custom service to allow users to use it is not easy.
You need to deploy your server, pay for resources, host it through a public address,
and to provide a client application that they can use to interact with your service.

This is a lot of work and maintenance.
Sometimes it can be justified, but sometimes the only purpose is to expose a simple service they can use,
and you do not want to deal with all these amount of work.

`mailfred` tries to give a solution to this using the current email infrastructure.
It reads emails from an email account, fetch them, process them, and reply them back to the remitter.
It's not act as an email server, it acts as a client (using SMTP and IMAP protocols) that connects to an email service provider.
You don't need to set up and deploy a server email.
You don't need to host anything or buy a public domain address to make it accesible
(you can run it from your own home if you want).
And more important, all your users already have your client application in their mobiles and computers:
their own email client applications that they already know how to use.

## How it works?
`mailfred` is inspired by [`axum`](https://github.com/tokio-rs/axum).
It works as an HTTP server, but instead of connecting through a *TCP transport* on *port 80*,
it is connected through *IMAP and SMTP protocols* on an *email address*.

Each email sent to that email address is fetched and interpreted as if it was an HTTP request.
The request email is routing to the correct service using the *subject*.
Once it is processed by the service, a new email is sent back to the remitter as if it was an HTTP response.

TODO: image

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
TODO
```rust,no-run

```

## Gmail account configuration
If you want to use a *Gmail* account, you need to set up some things before:
1. Create a new account, do NOT use your normal account. The *IMAP* service will remove the messages it reads from the inbox.
2. Enable *IMAP* in the *Gmail* configuration.
3. Enable [Gmail's app passwords](https://support.google.com/accounts/answer/185833?hl=en) for the account.
