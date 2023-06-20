use std::{
    net::{Shutdown, TcpStream},
    sync::Arc,
};

use async_trait::async_trait;
use imap::{
    types::{Flag, UnsolicitedResponse},
    ClientBuilder, Session,
};
use mail_parser::{HeaderValue, Message as EmailParser, MimeHeaders};
use native_tls::{TlsConnector, TlsStream};
use tokio::{
    runtime::Handle,
    sync::{mpsc, Notify},
};

use crate::{
    message::{Kind, Message, Part},
    transport::{Receiver, Transport},
};

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
                let err = listener(session, ready_to_recv.clone(), tx.clone())
                    .expect_err("listener only ends with error");

                tx.blocking_send(Err(err)).ok();
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
    loop {
        let fetches = session.fetch("1:*", "RFC822")?;

        for fetch in fetches.iter() {
            if fetch.flags().contains(&Flag::Deleted) {
                continue;
            }

            if let Some(body) = fetch.body() {
                if let Some(msg) = read_email(body) {
                    // We want to be sure we only remove the message
                    // if it will be processed.
                    let ready_to_recv = ready_to_recv.clone();
                    Handle::current().block_on(async move { ready_to_recv.notified().await });

                    tx.blocking_send(Ok(msg)).ok();

                    session.store(fetch.message.to_string(), "+FLAGS (\\Deleted)")?;
                }
            }
        }

        if fetches.len() > 0 {
            session.expunge()?;
        } else {
            // If a message is sent here, before initialize the IDLE,
            // the server could not notify it.
            // See issue: https://github.com/jonhoo/rust-imap/issues/263
            session.idle().wait_while(|response| match response {
                UnsolicitedResponse::Exists(_) => false,
                _ => true,
            })?;
        }
    }
}

fn read_email(email_raw: &[u8]) -> Option<Message> {
    let email = EmailParser::parse(email_raw)?;

    let subject = email.subject().unwrap_or_default().into();

    let from = match email.from() {
        HeaderValue::Address(addr) => addr.address.clone()?.into(),
        _ => None?,
    };

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
