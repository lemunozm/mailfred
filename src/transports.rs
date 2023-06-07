pub mod imap;
pub mod smtp;

pub use self::{imap::Imap, smtp::Smtp};
use crate::connector::Connector;

pub struct Gmail {
    pub username: String,
    pub password: String,
}

impl Connector for Gmail {
    type Inbound = Imap;
    type Outbound = Smtp;

    fn split(self) -> (Self::Inbound, Self::Outbound) {
        let imap = Imap {
            domain: "imap.gmail.com".into(),
            port: 993,
            user: format!("{}@gmail.com", self.username),
            password: self.password.clone(),
        };

        let smtp = Smtp {
            domain: "smtp.gmail.com".into(),
            port: 587,
            user: format!("{}@gmail.com", self.username),
            password: self.password,
        };

        (imap, smtp)
    }
}
