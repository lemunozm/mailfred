use mailfred::{
    self,
    message::{Message, Part},
    transports::Gmail,
};

async fn echo(msg: Message) -> Vec<Part> {
    msg.body.clone()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // log here

    let gmail = Gmail {
        username: "user".into(),
        password: "1234".into(),
    };

    mailfred::serve(gmail, echo).await
}
