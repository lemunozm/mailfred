use async_trait::async_trait;
use mail_builder::MessageBuilder as EmailBuilder;
use mail_send::{self as smtp, SmtpClient, SmtpClientBuilder};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

use crate::message::{Message, Sender, Transport};

pub struct Smtp {
    pub domain: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

#[async_trait]
impl Transport for Smtp {
    type Connection = SmtpConnection;
    type Error = smtp::Error;

    async fn connect(&self) -> smtp::Result<Self::Connection> {
        let client = SmtpClientBuilder::new(self.domain.as_ref(), self.port)
            .implicit_tls(false)
            .credentials((self.user.as_ref(), self.password.as_ref()))
            .connect()
            .await
            .unwrap();

        Ok(SmtpConnection {
            client,
            origin: self.user.clone(),
        })
    }
}

pub struct SmtpConnection {
    client: SmtpClient<TlsStream<TcpStream>>,
    origin: String,
}

#[async_trait]
impl Sender for SmtpConnection {
    type Error = smtp::Error;

    async fn send(&mut self, msg: &Message) -> smtp::Result<()> {
        let email = EmailBuilder::new()
            .from(self.origin.as_str())
            .to(msg.address.as_str())
            .subject(msg.subject.clone());

        self.client.send(email).await.unwrap();

        Ok(())
    }
}
