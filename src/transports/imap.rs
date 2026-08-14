use std::{
    collections::HashSet,
    net::{Shutdown, TcpStream},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use imap::{
    types::{Flag, Uid, UnsolicitedResponse},
    ClientBuilder, Session,
};
use mail_parser::{Addr, HeaderValue, Message as EmailParser, MimeHeaders};
use native_tls::{TlsConnector, TlsStream};
use tokio::{
    runtime::Handle,
    sync::{mpsc, Notify},
};

use crate::{
    message::{Kind, Message, Part},
    transport::{Receiver, Transport},
};

/// How often the `IDLE` command is refreshed.
///
/// RFC 2177 allows up to 29 minutes, which is the default of the `imap` crate,
/// but that period is also the read timeout of the underlying socket.
/// A connection that dies silently (the machine was suspended, the network
/// changed, a NAT dropped the entry) is indistinguishable from an idle one
/// until that timeout expires, so a long period means the listener keeps
/// waiting on a dead socket for half an hour before the error is reported and
/// the connection is recreated.
///
/// Refreshing often turns the keepalive into a liveness probe: every period the
/// client must write `DONE` and read the answer, which fails fast when the
/// connection is no longer usable.
const IDLE_REFRESH: Duration = Duration::from_secs(2 * 60);

/// Keyword used to mark the messages that can not be interpreted.
///
/// Such a message is not removed, because removing a message that was never
/// understood (and so, never answered) would lose it silently. Marking it in
/// the server instead means it is downloaded only once, and that the mark
/// survives reconnections and restarts.
const UNPROCESSABLE_KEYWORD: &str = "mailfred-unprocessable";

#[derive(Clone)]
pub struct Imap {
    pub domain: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub folder: String,
}

#[async_trait]
impl Transport for Imap {
    const NAME: &'static str = "imap";

    type Connection = ImapConnection;
    type Error = imap::Error;

    async fn connect(&self) -> imap::Result<ImapConnection> {
        let (session, tcp) = tokio::task::block_in_place(move || -> imap::Result<_> {
            let mut tcp_stream = None;
            let client = ClientBuilder::new(&self.domain, self.port).connect(|domain, tcp| {
                tcp_stream = Some(tcp.try_clone()?);
                let ssl_conn = TlsConnector::builder().build()?;
                Ok(TlsConnector::connect(&ssl_conn, domain, tcp)?)
            })?;

            let mut session = client
                .login(&self.user, &self.password)
                .map_err(|(e, _)| e)?;

            session.select(&self.folder)?;
            Ok((session, tcp_stream.expect("a session must have a stream")))
        })?;

        let ready_to_recv = Arc::new(Notify::new());
        let (tx, rx) = mpsc::channel(1);

        tokio::task::spawn_blocking({
            let ready_to_recv = ready_to_recv.clone();
            move || {
                // The listener ends either with an error, that must be
                // notified to trigger a reconnection, or because the
                // `ImapConnection` is gone, where there is nobody to notify.
                if let Err(err) = listener(session, ready_to_recv, tx.clone()) {
                    tx.blocking_send(Err(err)).ok();
                }
            }
        });

        Ok(ImapConnection {
            rx,
            tcp,
            ready_to_recv,
        })
    }
}

fn listener(
    mut session: Session<TlsStream<TcpStream>>,
    ready_to_recv: Arc<Notify>,
    tx: mpsc::Sender<imap::Result<Message>>,
) -> imap::Result<()> {
    // Messages that could not be parsed are left in the folder: we do not want
    // to remove a message we did not understand. They must not be retried
    // forever though, or the folder would never look empty and the IDLE branch
    // below would be unreachable, turning this loop into a busy loop that
    // downloads the whole folder over and over again.
    //
    // The mark is kept in the server, so a message is downloaded at most once
    // no matter how many times the server is restarted or reconnected.
    // This set only backs up the servers that do not support custom keywords.
    let mut unprocessable = HashSet::new();
    let unprocessable_flag = Flag::Custom(UNPROCESSABLE_KEYWORD.into());

    loop {
        // Only the flags are asked here. The folder can contain messages
        // already marked as deleted, or messages that can not be handled,
        // and downloading their full content on every pass is expensive.
        // Note also that UIDs are used instead of sequence numbers: the latter
        // are renumbered whenever any client expunges the folder, which would
        // make us flag as deleted a message different from the one just read.
        // The attribute list must be parenthesized: more than one is asked.
        let fetches = session.uid_fetch("1:*", "(UID FLAGS)")?;

        // A message can be already flagged as deleted if the connection fell
        // before the folder was expunged, so the folder is cleaned up here too.
        let mut expunge_required = fetches
            .iter()
            .any(|fetch| fetch.flags().contains(&Flag::Deleted));

        let pending = fetches
            .iter()
            .filter(|fetch| {
                let flags = fetch.flags();
                !flags.contains(&Flag::Deleted) && !flags.contains(&unprocessable_flag)
            })
            .filter_map(|fetch| fetch.uid)
            .filter(|uid| !unprocessable.contains(uid))
            .collect::<Vec<_>>();

        for uid in &pending {
            let uid = *uid;

            let Some(msg) = fetch_email(&mut session, uid)? else {
                log::warn!(
                    "imap: message with uid {} can not be read, marking it to \
                     not be processed again",
                    uid
                );

                // Not all the servers accept custom keywords. If the mark can
                // not be stored, the message is only skipped for this
                // connection, and it will be downloaded again by the next one.
                let stored = session.uid_store(
                    uid.to_string(),
                    format!("+FLAGS ({})", UNPROCESSABLE_KEYWORD),
                );

                if let Err(err) = stored {
                    log::debug!("imap: the folder does not accept keywords: {}", err);
                }

                unprocessable.insert(uid);
                continue;
            };

            // We want to be sure we only remove the message
            // if it will be processed.
            let ready_to_recv = ready_to_recv.clone();
            Handle::current().block_on(async move { ready_to_recv.notified().await });

            if tx.blocking_send(Ok(msg)).is_err() {
                // Nobody will process it, so it must stay in the folder to be
                // read again by the next connection.
                return Ok(());
            }

            session.uid_store(uid.to_string(), "+FLAGS (\\Deleted)")?;
            expunge_required = true;
        }

        if expunge_required {
            session.expunge()?;
        }

        if pending.is_empty() {
            // If a message is sent here, before initialize the IDLE,
            // the server could not notify it.
            // See issue: https://github.com/jonhoo/rust-imap/issues/263
            //
            // `keepalive` is left enabled on purpose: it keeps the read
            // timeout armed while the IDLE is refreshed, so a dead connection
            // reports an error instead of blocking forever.
            session
                .idle()
                .timeout(IDLE_REFRESH)
                .wait_while(|response| {
                    !matches!(
                        response,
                        UnsolicitedResponse::Exists(_) | UnsolicitedResponse::Recent(_)
                    )
                })?;
        }
    }
}

/// Downloads a single message.
/// Returns `None` if the message is gone or can not be interpreted.
fn fetch_email(
    session: &mut Session<TlsStream<TcpStream>>,
    uid: Uid,
) -> imap::Result<Option<Message>> {
    let fetches = session.uid_fetch(uid.to_string(), "RFC822")?;

    Ok(fetches
        .iter()
        .find_map(|fetch| fetch.body())
        .and_then(read_email))
}

/// Extracts the first usable address of an address header.
/// A `From` header usually contains a single address, but a list of addresses
/// or a group are also valid, and they must not be rejected: a message whose
/// remitter can not be found is a message that can never be answered.
fn read_address(header: &HeaderValue) -> Option<String> {
    fn first(addrs: &[Addr]) -> Option<String> {
        addrs
            .iter()
            .find_map(|addr| addr.address.as_deref())
            .map(Into::into)
    }

    match header {
        HeaderValue::Address(addr) => addr.address.as_deref().map(Into::into),
        HeaderValue::AddressList(addrs) => first(addrs),
        HeaderValue::Group(group) => first(&group.addresses),
        HeaderValue::GroupList(groups) => groups.iter().find_map(|group| first(&group.addresses)),
        _ => None,
    }
}

fn read_email(email_raw: &[u8]) -> Option<Message> {
    let email = EmailParser::parse(email_raw)?;

    let subject = email.subject().unwrap_or_default().into();

    let from = read_address(email.from())?;

    let mut body = Vec::default();

    for part in email.text_bodies() {
        body.push(Part {
            kind: if part.is_text_html() {
                Kind::Html
            } else {
                Kind::Text
            },
            content: part.contents().into(),
        });
    }

    for part in email.attachments() {
        if !part.is_empty() {
            body.push(Part {
                kind: Kind::Attachment(part.attachment_name().unwrap_or_default().into()),
                content: part.contents().into(),
            });
        }
    }

    Some(Message {
        address: from,
        header: subject,
        body,
    })
}

impl Imap {
    pub fn clear_folder(&self, folder: &str) -> imap::Result<()> {
        let client = imap::ClientBuilder::new(&self.domain, self.port).native_tls()?;
        let mut session = client.login(&self.user, &self.password).map_err(|e| e.0)?;

        session.select(folder)?;
        session.store("1:*", "+FLAGS (\\Deleted)")?;
        session.expunge()?;

        Ok(())
    }
}

pub struct ImapConnection {
    rx: mpsc::Receiver<imap::Result<Message>>,
    tcp: TcpStream,
    ready_to_recv: Arc<Notify>,
}

#[async_trait]
impl Receiver for ImapConnection {
    type Error = imap::Error;

    async fn recv(&mut self) -> imap::Result<Message> {
        self.ready_to_recv.notify_one();
        match self.rx.recv().await {
            Some(message) => message,
            None => unreachable!(),
        }
    }
}

impl Drop for ImapConnection {
    fn drop(&mut self) {
        self.tcp.shutdown(Shutdown::Both).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn email(headers: &str) -> Option<Message> {
        read_email(format!("{headers}\r\n\r\nbody\r\n").as_bytes())
    }

    fn remitter(headers: &str) -> Option<String> {
        email(headers).map(|msg| msg.address)
    }

    #[test]
    fn remitter_of_a_single_address() {
        assert_eq!(remitter("From: a@b.com").as_deref(), Some("a@b.com"));
        assert_eq!(remitter("From: Bob <a@b.com>").as_deref(), Some("a@b.com"));
    }

    /// A `From` header is not always a single address. Any of these forms
    /// used to be rejected, and a rejected message stayed in the folder
    /// forever, preventing the listener from ever reaching the IDLE state.
    #[test]
    fn remitter_of_a_list_or_a_group() {
        assert_eq!(
            remitter("From: a@b.com, c@d.com").as_deref(),
            Some("a@b.com")
        );
        assert_eq!(
            remitter("From: Team: a@b.com, c@d.com;").as_deref(),
            Some("a@b.com")
        );
    }

    #[test]
    fn message_without_a_usable_remitter_is_discarded() {
        assert_eq!(remitter("Subject: no from header"), None);
        assert_eq!(remitter("From: "), None);
        assert_eq!(remitter("From: undisclosed-recipients:;"), None);
    }

    #[test]
    fn subject_is_read_as_the_header() {
        let msg = email("From: a@b.com\r\nSubject: Count").unwrap();
        assert_eq!(msg.header, "Count");

        // A message without subject is still routable, through an empty header
        let msg = email("From: a@b.com").unwrap();
        assert_eq!(msg.header, "");
    }
}
