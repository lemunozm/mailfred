#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Text,
    Html,
    Attachment(String),
}

impl Kind {
    pub fn attachment_name(&self) -> &str {
        match self {
            Kind::Attachment(name) => name,
            _ => panic!("Kind is not an attachment"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub kind: Kind,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub address: String,
    pub header: String,
    pub body: Vec<Part>,
}

impl Message {
    pub fn text_iter(&self) -> impl Iterator<Item = &Part> {
        self.body.iter().filter(|part| part.kind == Kind::Text)
    }

    pub fn html_iter(&self) -> impl Iterator<Item = &Part> {
        self.body.iter().filter(|part| part.kind == Kind::Html)
    }

    pub fn attachment_iter(&self) -> impl Iterator<Item = &Part> {
        self.body
            .iter()
            .filter(|part| matches!(part.kind, Kind::Attachment(_)))
    }
}
