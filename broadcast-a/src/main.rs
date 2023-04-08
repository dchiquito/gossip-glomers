use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

mod server;
use crate::server::Server;

#[derive(Serialize, Deserialize, Debug)]
struct GenerateOk {
    r#type: String,
    msg_id: u64,
    id: u64,
}

fn main() -> serde_json::Result<()> {
    let mut server = Server::new();
    let mut hasher = DefaultHasher::new();
    server.sender.node_id.hash(&mut hasher);
    let server_hash = hasher.finish();
    let mut counter: u64 = 1;
    server.handle("generate", move |sender, msg| {
        let id = server_hash + counter;
        counter += 1;
        let generate_ok = GenerateOk {
            r#type: "generate_ok".to_string(),
            msg_id: id,
            id,
        };
        sender.respond(msg, &generate_ok)
    });
    server.serve()
}
