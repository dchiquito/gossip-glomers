use std::collections::HashSet;
use std::{cell::RefCell, rc::Rc};

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
struct Read {
    r#type: String,
}
impl Read {
    fn new() -> Read {
        Read {
            r#type: "read".to_string(),
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
    let sender = server.sender.clone();
    let adj_nodes_thread = sender.lock().unwrap().node_ids.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        for adj_node in adj_nodes_thread.iter() {
            sender
                .lock()
                .unwrap()
                .send_body(adj_node, &Read::new())
                .expect("Error sending refresh");
        }
    });
    let adj_nodes = server.sender.lock().unwrap().node_ids.clone();
    let messages: Rc<RefCell<Vec<u64>>> = Rc::new(RefCell::new(vec![]));
    let messages_read = messages.clone();
    let messages_read_ok = messages.clone();
    server.handle("broadcast", move |sender, msg| {
        let msg: Message<Broadcast> = msg.cast()?;
        let message = msg.body.message;
        let mut messages = messages.borrow_mut();
        if !messages.contains(&message) {
            messages.push(message);
            for adj_node in adj_nodes.iter() {
                sender.send_body(adj_node, &Broadcast::new(message))?;
            }
        }
        sender.respond(&msg, &BroadcastOk::new())
    });
    server.handle("broadcast_ok", move |_, _| Ok(()));
    server.handle("read", move |sender, msg| {
        sender.respond(msg, &ReadOk::new(messages_read.borrow().to_vec()))
    });
    server.handle("read_ok", move |_, msg| {
        let msg: Message<ReadOk> = msg.cast()?;
        for value in msg.body.messages {
            let mut messages = messages_read_ok.borrow_mut();
            if !messages.contains(&value) {
                messages.push(value);
            }
        }
        Ok(())
    });
    server.handle("topology", move |sender, msg| {
        // Just ignore the topology lmao
        sender.respond(msg, &TopologyOk::new())
    });
    server.serve()
}
