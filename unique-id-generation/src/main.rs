use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use server::Message;

mod server;
use crate::server::Server;

#[derive(Serialize, Deserialize, Debug)]
struct Generate {
    r#type: String,
    msg_id: u64,
    in_reply_to: Option<u32>,
    echo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenerateOk {
    r#type: String,
    msg_id: u64,
    id: u64,
}

fn main() -> serde_json::Result<()> {
    let server = Server::new();
    let mut hasher = DefaultHasher::new();
    server.node_id.hash(&mut hasher);
    let server_hash = hasher.finish();
    let mut counter: u64 = 1;
    loop {
        let generate: Message<Generate> = server.read_message()?;
        let id = server_hash + counter;
        counter += 1;
        let generate_ok = GenerateOk {
            r#type: "generate_ok".to_string(),
            msg_id: id,
            id,
        };
        server.respond(&generate, &generate_ok)?;
    }
}
