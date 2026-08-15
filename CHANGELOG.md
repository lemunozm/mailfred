# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-08-15

### Fixed

- **The IMAP listener could stop waiting for new emails and start looping over
  the whole folder.** A message whose remitter could not be read was never
  removed, so the folder never looked empty, and the listener kept downloading
  every message again and again instead of reaching its `IDLE` state. These
  messages are now marked in the server with the `mailfred-unprocessable`
  keyword and skipped, so each one is downloaded at most once.
- **Emails coming from more than one address were rejected.** Only a `From`
  header with a single address was accepted, so a list of addresses or a group
  made the message unreadable. The first usable address is taken now.
- **A message different from the one just read could be removed.** Messages
  were addressed by sequence number, which the server renumbers whenever any
  client expunges the folder. They are addressed by UID now.
- **A message could be removed without being processed**, if the service was
  not there to receive it. It is now kept in the folder to be read again by the
  next connection.
- **A connection lost silently was not noticed for almost half an hour**, for
  example after suspending the machine or changing of network. The `IDLE`
  command is refreshed every 2 minutes instead of every 29, and that period is
  also the read timeout of the socket, so a dead connection is detected in
  minutes and reconnected.
- **Reading the folder was slower than needed**: the full content of every
  message, attachments included, was downloaded on every pass, even for
  messages already marked as deleted. Only the flags are downloaded now, and
  the content only for the messages to process.
- **Reconnections waited longer on each attempt instead of less.** The backoff
  was capped from below rather than from above, so it never waited less than 60
  seconds, grew without bound, and eventually overflowed. It now starts at one
  second and is capped at 60.
- The log warning about a long disconnection printed its values swapped.
- The documentation example of `util::logger::configure` did not compile.

### Changed

- `ResponseBody` implements `Display` instead of `ToString`. Calls to
  `to_string()` keep working through the blanket implementation of the standard
  library.
- Dependencies updated: `mail-send` 0.4 → 0.6, `mail-builder` 0.3 → 0.4,
  `tokio-rustls` 0.24 → 0.26 and `imap` 3.0.0-alpha.10 → 3.0.0-alpha.15.
- The `dkim` feature of `mail-send` is no longer enabled. mailfred does not
  sign the messages it sends, and that feature brought a whole DNS resolver
  with it. Its `ring` backend is used instead of the default `aws-lc-rs` to
  keep a C toolchain out of the build.

### Security

- Updated the locked dependencies, closing the 26 advisories reported against
  them, affecting `openssl`, `rustls`, `rustls-webpki`, `ring`, `mio`, `tokio`,
  `bytes`, `rand`, `time`, `idna`, `hickory-proto` and `ouroboros`.

## [0.1.1] - 2024-02-13

### Fixed

- Pinned `mail-send` and `imap` to exact versions, as newer releases of both
  broke the build.

### Added

- `Gmail::new()`, to build the connector without naming its fields.
- Documentation of the main API.
- Continuous integration checking format, clippy and tests.

## [0.1.0] - 2023-06-22

First release.

### Added

- `serve()`, running a service over an email account.
- `Message`, with text, HTML and attachment parts.
- `Service`, implemented by any async function taking a request and a state.
- `Router`, dispatching by email subject, with `Filter` and `Layer` extension
  points.
- IMAP and SMTP transports, and the `Gmail` connector pairing them.
- Optional `logger` utility.

[0.1.2]: https://github.com/lemunozm/mailfred/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/lemunozm/mailfred/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/lemunozm/mailfred/releases/tag/v0.1.0
