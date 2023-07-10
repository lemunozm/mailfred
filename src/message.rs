use std::str::{self, Utf8Error};

/// Define the type of a message part
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// Plain text section of a message
    Text,
    /// HTML section of a message
    Html,
    /// Attachement with name
    Attachment(String),
}

impl Kind {
    /// Assuming it's an attachment, it retrieves the name of it.
    pub fn attachment_name(&self) -> &str {
        match self {
            Kind::Attachment(name) => name,
            _ => panic!("Kind is not an attachment"),
        }
    }
}

/// Represents a part of a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub kind: Kind,
    pub content: Vec<u8>,
}

impl Part {
    /// Transform the byte content into a readable utf8 string
    pub fn as_utf8(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(&self.content)
    }
}

/// Represents a message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Remote address where the message comes from or goes to
    pub address: String,
    /// Represents the main header or subject of a message
    pub header: String,
    /// Contains the information of a message
    pub body: Vec<Part>,
}

impl Message {
    /// Iterates over all text parts
    pub fn text_iter(&self) -> impl Iterator<Item = &Part> {
        self.body.iter().filter(|part| part.kind == Kind::Text)
    }

    /// Iterates over all html parts
    pub fn html_iter(&self) -> impl Iterator<Item = &Part> {
        self.body.iter().filter(|part| part.kind == Kind::Html)
    }

    /// Iterates over all attachments
    pub fn attachment_iter(&self) -> impl Iterator<Item = &Part> {
        self.body
            .iter()
            .filter(|part| matches!(part.kind, Kind::Attachment(_)))
    }
}
