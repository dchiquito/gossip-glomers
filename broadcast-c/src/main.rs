use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use server::{Message, Server};

mod server;

#[derive(Serialize, Deserialize, Debug)]
struct Broadcast {
    r#type: String,
    msg_id: Option<u64>,
    message: u64,
}
impl Broadcast {
    fn new(message: u64) -> Broadcast {
        Broadcast {
            r#type: "broadcast".to_string(),
            msg_id: None,
            message,
        }
    }
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
    let adj_nodes = server.sender.node_ids.clone();
    let messages = Rc::new(RefCell::new(vec![]));
    let messages_read = messages.clone();
    let mut messages_set = HashSet::new();
    server.handle("broadcast", move |sender, msg| {
        let msg: Message<Broadcast> = msg.cast()?;
        let message = msg.body.message;
        if messages_set.insert(message) {
            messages.borrow_mut().push(message);
            for adj_node in adj_nodes.iter() {
                sender.rpc(adj_node, &Broadcast::new(message), |_, _| Ok(()))?;
            }
        }
        sender.respond(&msg, &BroadcastOk::new())
    });
    server.handle("read", move |sender, msg| {
        sender.respond(msg, &ReadOk::new(messages_read.borrow().clone()))
    });
    server.handle("topology", move |sender, msg| {
        // Just ignore the topology lmao
        sender.respond(msg, &TopologyOk::new())
    });
    server.serve()
}
