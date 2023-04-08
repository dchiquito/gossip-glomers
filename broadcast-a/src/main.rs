use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use server::{Message, Server};

mod server;

#[derive(Serialize, Deserialize, Debug)]
struct Broadcast {
    r#type: String,
    msg_id: u64,
    message: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastOk {
    r#type: String,
}
impl BroadcastOk {
    fn new() -> BroadcastOk {
        BroadcastOk {
            r#type: "broadcast_ok".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadOk {
    r#type: String,
    messages: Vec<u64>,
}
impl ReadOk {
    fn new(messages: Vec<u64>) -> ReadOk {
        ReadOk {
            r#type: "read_ok".to_string(),
            messages,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TopologyOk {
    r#type: String,
}
impl TopologyOk {
    fn new() -> TopologyOk {
        TopologyOk {
            r#type: "topology_ok".to_string(),
        }
    }
}

fn main() -> serde_json::Result<()> {
    let mut server = Server::new();
    let messages = Rc::new(RefCell::new(vec![]));
    let messages_read = messages.clone();
    server.handle("broadcast", move |sender, msg| {
        let msg: Message<Broadcast> = msg.cast()?;
        let message = msg.body.message;
        messages.borrow_mut().push(message);
        sender.respond(&msg, &BroadcastOk::new())
    });
    server.handle("read", move |sender, msg| {
        sender.respond(msg, &ReadOk::new(messages_read.borrow().clone()))
    });
    server.handle("topology", move |sender, msg| {
        sender.respond(msg, &TopologyOk::new())
    });
    server.serve()
}
