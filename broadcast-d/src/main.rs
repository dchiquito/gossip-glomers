use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use server::Message;

mod server;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum P {
    Broadcast {
        #[serde(rename = "message")]
        value: u64,
    },
    BroadcastOk {},
    Read {},
    ReadOk {
        #[serde(rename = "messages")]
        values: Vec<u64>,
    },
    Topology {},
    TopologyOk {},
}

fn main() -> serde_json::Result<()> {
    let (server, sender) = server::init()?;
    let mut values = vec![];
    let adj_nodes = sender.node_ids.clone();
    let adj_nodes_thread = adj_nodes.clone();
    let sender = Arc::new(Mutex::new(sender));
    let thread_sender = sender.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        for adj_node in adj_nodes_thread.iter() {
            thread_sender
                .lock()
                .unwrap()
                .send(adj_node, &P::Read {})
                .expect("Error sending refresh");
        }
    });
    loop {
        let message: Message<P> = server.read_message()?;
        match message.body.fields {
            P::Broadcast { value } => {
                if !values.contains(&value) {
                    values.push(value);
                    for adj_node in adj_nodes.iter() {
                        sender
                            .lock()
                            .unwrap()
                            .send(adj_node, &P::Broadcast { value })?;
                    }
                }
                sender
                    .lock()
                    .unwrap()
                    .respond(&message, &P::BroadcastOk {})?
            }
            P::BroadcastOk {} => {}
            P::Read {} => sender.lock().unwrap().respond(
                &message,
                &P::ReadOk {
                    values: values.clone(),
                },
            )?,
            P::ReadOk {
                values: read_values,
            } => {
                for value in read_values {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                }
            }
            P::Topology {} => sender
                .lock()
                .unwrap()
                .respond(&message, &P::TopologyOk {})?,
            P::TopologyOk {} => {}
        }
    }
}
