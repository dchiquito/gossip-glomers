use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use server::{Message, Sender};

mod server;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum P {
    Add {
        delta: u64,
    },
    AddOk {},
    Read {
        key: Option<String>,
    },
    ReadOk {
        value: u64,
    },
    Write {
        key: String,
        value: u64,
    },
    WriteOk {},
    Cas {
        key: String,
        from: u64,
        to: u64,
        create_if_not_exists: bool,
    },
    CasOk {},
    Error {
        code: u64,
        text: String,
    },
}

struct Context {
    sender: Sender,
    delta: u64,
    last_global: u64,
    pendings: HashMap<u64, (u64, u64)>,
}

fn main() -> serde_json::Result<()> {
    let (server, mut sender) = server::init()?;
    sender.send(
        "seq-kv",
        &P::Write {
            key: "global".to_string(),
            value: 0,
        },
    )?;
    let context: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context {
        sender,
        delta: 0,
        last_global: 0,
        pendings: HashMap::new(),
    }));
    let thread_context = context.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut ctx = thread_context.lock().unwrap();
        let ctx = ctx.deref_mut();
        // Reread the global value periodically for eventual consistency
        ctx.sender
            .send(
                "seq-kv",
                &P::Read {
                    key: Some("global".to_string()),
                },
            )
            .expect("Error sending reread");
    });
    loop {
        let message: Message<P> = server.read_message()?;
        let mut ctx = context.lock().unwrap();
        let mut ctx = ctx.deref_mut();
        match message.body.fields {
            // Increment our local delta appropriately
            P::Add { delta } => {
                ctx.delta += delta;
                ctx.sender
                    .send(
                        "seq-kv",
                        &P::Read {
                            key: Some("global".to_string()),
                        },
                    )
                    .expect("Error sending reread");
                ctx.sender.respond(&message, &P::AddOk {})?
            }
            // Maelstrom wants to know what we think the global is, use the last_global
            P::Read { key: None } => ctx.sender.respond(
                &message,
                &P::ReadOk {
                    value: ctx.last_global,
                },
            )?,
            // A read from the seq-kv has returned!
            // Update our last_global, and send off a nice fresh CAS
            P::ReadOk { value } => {
                ctx.last_global = value;
                if ctx.delta > 0 {
                    let expected_value = ctx.last_global + ctx.delta;
                    let request = &P::Cas {
                        key: "global".to_string(),
                        from: ctx.last_global,
                        to: expected_value,
                        create_if_not_exists: true,
                    };
                    let message = ctx.sender.message("seq-kv", request)?;
                    ctx.pendings.insert(
                        message.body.msg_id.expect("No msg_id???"),
                        (ctx.delta, expected_value),
                    );
                    ctx.sender.send_message(&message)?;
                }
            }
            P::WriteOk {} => {}
            // A CAS has succeeded!
            // Look up which request succeeded and adjust delta and last_global accordingly.
            P::CasOk {} => {
                let (old_delta, old_expected_value) = *ctx
                    .pendings
                    .get(&message.body.in_reply_to.unwrap())
                    .unwrap();
                ctx.pendings.clear();
                ctx.delta -= old_delta;
                ctx.last_global = old_expected_value;
            }
            P::Error { code, text } => {
                if code == 20 {
                    eprintln!("ERROR: {} {}", code, text);
                } else if code == 22 {
                    eprintln!("No cause for alarm, we are simply out of sync");
                    ctx.sender
                        .send(
                            "seq-kv",
                            &P::Read {
                                key: Some("global".to_string()),
                            },
                        )
                        .expect("Error sending reread");
                } else {
                    panic!("DISASTER {} {}", code, text);
                }
            }
            _ => panic!("NOT ALLOWED"),
        }
    }
}
