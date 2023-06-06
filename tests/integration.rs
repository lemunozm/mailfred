use std::time::Duration;

use mailfred::{
    connections::{imap::Imap, smtp::Smtp},
    message::{Message, Receiver, Sender, Transport},
};

mod env {
    use std::env;

    pub fn user() -> String {
        env::var("MAILFRED_TEST_USER").unwrap()
    }

    pub fn password() -> String {
        env::var("MAILFRED_TEST_PASSWORD").unwrap()
    }
}

fn imap_transport() -> Imap {
    Imap {
        domain: "imap.gmail.com".into(),
        port: 993,
        user: env::user(),
        password: env::password(),
    }
}

fn smtp_transport() -> Smtp {
    Smtp {
        domain: "smtp.gmail.com".into(),
        port: 587,
        user: env::user(),
        password: env::password(),
    }
}

fn clean_inbox() -> imap::Result<()> {
    let client = imap::ClientBuilder::new("imap.gmail.com", 993).native_tls()?;
    let mut session = client
        .login(env::user(), env::password())
        .map_err(|e| e.0)?;

    session.select("INBOX")?;
    session.store("1:*", "+FLAGS (\\Deleted)")?;
    session.expunge()?;

    Ok(())
}

fn messages() -> Vec<Message> {
    vec![
        Message {
            address: env::user(),
            subject: "".into(),
            body: Vec::default(),
        },
        Message {
            address: env::user(),
            subject: "hello".into(),
            body: Vec::default(),
        },
    ]
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn roundtrip_sync() {
    clean_inbox().unwrap();

    let mut smtp = smtp_transport().connect().await.unwrap();
    for msg in messages() {
        smtp.send(&msg).await.unwrap();
    }

    // Start reading once all messages are in the server
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut imap = imap_transport().connect().await.unwrap();

    let messages = messages();
    for i in 0..messages.len() {
        let msg = imap.recv().await.unwrap();
        assert_eq!(messages[i], msg, "Message {i}");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn roundtrip_async() {
    clean_inbox().unwrap();

    let mut smtp = smtp_transport().connect().await.unwrap();
    let mut imap = imap_transport().connect().await.unwrap();

    // Wait to ensure the connections are ready
    tokio::time::sleep(Duration::from_secs(3)).await;

    tokio::spawn(async move {
        for msg in messages() {
            smtp.send(&msg).await.unwrap();
        }
    });

    let messages = messages();
    for i in 0..messages.len() {
        let msg = imap.recv().await.unwrap();
        assert_eq!(messages[i], msg, "Message {i}");
    }
}
