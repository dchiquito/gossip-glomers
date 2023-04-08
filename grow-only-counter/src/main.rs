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
    Add { delta: u64 },
    AddOk {},
    Read {},
    ReadOk { value: u64 },
    Error { code: u64, text: String },
}

struct Context {
    sender: Sender,
    value: u64,
}

fn main() -> serde_json::Result<()> {
    let (server, sender) = server::init()?;
    let neighbors = sender.node_ids.clone();
    let context: Arc<Mutex<Context>> = Arc::new(Mutex::new(Context { sender, value: 0 }));
    let thread_context = context.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let mut ctx = thread_context.lock().unwrap();
        let ctx = ctx.deref_mut();
        for neighbor in neighbors.iter() {
            ctx.sender
                .send(neighbor, &P::ReadOk { value: 666 })
                .expect("Error sending refresh");
        }
    });
    loop {
        let message: Message<P> = server.read_message()?;
        let mut ctx = context.lock().unwrap();
        let mut ctx = ctx.deref_mut();
        match message.body.fields {
            P::Add { delta } => {
                ctx.value += delta;
                ctx.sender.send("seq-kv", &P::Read {})?;
                ctx.sender.respond(&message, &P::AddOk {})?
            }
            P::AddOk {} => {}
            P::Read {} => ctx
                .sender
                .respond(&message, &P::ReadOk { value: ctx.value })?,
            P::ReadOk { value } => {
                panic!("ahhh {}", value)
            }
            P::Error { code, text } => panic!("DISASTER {} {}", code, text),
        }
    }
}
