use serde::{Deserialize, Serialize};
use server::Message;

mod server;
use crate::server::Server;

#[derive(Serialize, Deserialize, Debug)]
struct Echo {
    r#type: String,
    msg_id: u32,
    in_reply_to: Option<u32>,
    echo: Option<String>,
}

fn main() -> serde_json::Result<()> {
    let server = Server::new();
    loop {
        let mut message: Message<Echo> = server.read_message()?;
        message.body.r#type = "echo_ok".to_string();
        server.respond(&message, &message.body)?;
    }
}
