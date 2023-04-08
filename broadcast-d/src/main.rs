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
    Broadcast {
        #[serde(rename = "message")]
        value: u64,
    },
    BroadcastOk {},
    BroadcastToPeers {
        #[serde(rename = "message")]
        value: u64,
    },
    Read {},
    ReadOk {
        #[serde(rename = "messages")]
        values: Vec<u64>,
    },
    Fyi {
        #[serde(rename = "messages")]
        values: Vec<u64>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {},
}

type Context = (Sender, Vec<u64>, Vec<String>);

fn main() -> serde_json::Result<()> {
    let (server, sender) = server::init()?;
    let context: Arc<Mutex<Context>> = Arc::new(Mutex::new((sender, vec![], vec![])));
    let thread_context = context.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let mut ctx = thread_context.lock().unwrap();
        let (sender, values, neighbors) = ctx.deref_mut();
        for neighbor in neighbors {
            sender
                .send(
                    neighbor,
                    &P::Fyi {
                        values: values.clone(),
                    },
                )
                .expect("Error sending refresh");
        }
    });
    loop {
        let message: Message<P> = server.read_message()?;
        let mut ctx = context.lock().unwrap();
        let (sender, values, neighbors) = ctx.deref_mut();
        match message.body.fields {
            P::Broadcast { value } => {
                if !values.contains(&value) {
                    values.push(value);
                    for neighbor in neighbors.iter().filter(|n| n != &&message.src) {
                        sender.send(neighbor, &P::BroadcastToPeers { value })?;
                    }
                }
                sender.respond(&message, &P::BroadcastOk {})?
            }
            P::BroadcastOk {} => {}
            P::BroadcastToPeers { value } => {
                if !values.contains(&value) {
                    values.push(value);
                    for neighbor in neighbors.iter().filter(|n| n != &&message.src) {
                        sender.send(neighbor, &P::BroadcastToPeers { value })?;
                    }
                }
            }
            P::Read {} => sender.respond(
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
            P::Fyi {
                values: read_values,
            } => {
                for value in read_values {
                    if !values.contains(&value) {
                        values.push(value);
                    }
                }
            }
            P::Topology { ref topology } => {
                let new_neighbors = topology
                    .get(&sender.node_id)
                    .expect("This node is not in the topology");
                neighbors.clear();
                for neighbor in new_neighbors {
                    neighbors.push(neighbor.to_string());
                }
                sender.respond(&message, &P::TopologyOk {})?
            }
            P::TopologyOk {} => {}
        }
    }
}
