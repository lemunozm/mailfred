use std::time::Duration;

use mailfred::{
    message::{Kind, Message, Part},
    service::Response,
    transport::{Receiver, Sender, Transport},
    transports::{Imap, Smtp},
};

mod env {
    use std::env;

    pub fn user() -> String {
        env::var("MAILFRED_TEST_USER").expect("MAILFRED_TEST_USER environment variable")
    }

    pub fn password() -> String {
        env::var("MAILFRED_TEST_PASSWORD").expect("MAILFRED_TEST_PASSWORD environment variable")
    }
}

fn imap_transport() -> Imap {
    Imap {
        domain: "imap.gmail.com".into(),
        port: 993,
        user: env::user(),
        password: env::password(),
        folder: "inbox".into(),
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

fn messages() -> Vec<Message> {
    vec![
        Message {
            address: env::user(),
            header: "".into(),
            body: Vec::default(),
        },
        Message {
            address: env::user(),
            header: "Empty message".into(),
            body: Vec::default(),
        },
        Message {
            address: env::user(),
            header: "Text message".into(),
            body: vec![Part {
                kind: Kind::Text,
                content: "asd".as_bytes().into(),
            }],
        },
        Message {
            address: env::user(),
            header: "Html message".into(),
            body: vec![Part {
                kind: Kind::Html,
                content: "<h1>abc</h1>".as_bytes().into(),
            }],
        },
        Message {
            address: env::user(),
            header: "Attachment message".into(),
            body: vec![Part {
                kind: Kind::Attachment("file.txt".into()),
                content: "file content".as_bytes().into(),
            }],
        },
        Message {
            address: env::user(),
            header: "Complex message".into(),
            body: vec![
                Part {
                    kind: Kind::Text,
                    content: "asd 1".as_bytes().into(),
                },
                Part {
                    kind: Kind::Text,
                    content: "asd 2".as_bytes().into(),
                },
                Part {
                    kind: Kind::Html,
                    content: "<h1>abc</h1>".as_bytes().into(),
                },
                Part {
                    kind: Kind::Attachment("file1.txt".into()),
                    content: "file content 1".as_bytes().into(),
                },
                Part {
                    kind: Kind::Attachment("file2.txt".into()),
                    content: "file content 2".as_bytes().into(),
                },
            ],
        },
    ]
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn roundtrip_sync() {
    imap_transport().clear_folder("inbox").unwrap();

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
        assert_eq!(msg, messages[i], "Message {i}");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn roundtrip_async() {
    imap_transport().clear_folder("inbox").unwrap();

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
        assert_eq!(msg, messages[i], "Message {i}");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn run_and_stop() {
    let connector = (imap_transport(), smtp_transport());
    let handle = tokio::spawn(async move {
        mailfred::serve(connector, (), |_, _| async {
            Response::ok("run_and_stop", ())
        })
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_secs(5)).await;

    handle.abort();
}

#[ignore] // Used only for manual testing
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial]
async fn send() {
    let msg = Message {
        address: env::user(),
        header: "Hi".into(),
        body: vec![
            Part {
                kind: Kind::Text,
                content: "asdasd".as_bytes().into(),
            },
            Part {
                kind: Kind::Attachment("file.txt".into()),
                content: "file content".as_bytes().into(),
            },
        ],
    };

    let mut smtp = smtp_transport().connect().await.unwrap();
    smtp.send(&msg).await.unwrap();
}
