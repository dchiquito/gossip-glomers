use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{to_value, Value};

#[derive(Serialize, Deserialize, Debug)]
struct Message<T> {
    src: String,
    dest: String,
    body: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct Init {
    r#type: String,
    msg_id: u32,
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InitOk {
    r#type: String,
    in_reply_to: u32,
}

impl InitOk {
    fn new(init: &Init) -> InitOk {
        InitOk {
            r#type: "init_ok".to_string(),
            in_reply_to: init.msg_id,
        }
    }
}

struct Server {
    node_id: String,
    node_ids: Vec<String>,
}

impl Server {
    fn new() -> Server {
        let mut server = Server {
            node_id: "".to_string(),
            node_ids: vec![],
        };
        let init: Message<Init> = server.read_message().unwrap();
        let init_ok = InitOk::new(&init.body);
        server.node_id = init.body.node_id.clone();
        server.node_ids = init.body.node_ids.clone();
        server.respond(&init, &init_ok).unwrap();
        server
    }
    fn read_message<T: DeserializeOwned>(&self) -> serde_json::Result<Message<T>> {
        let stdin = std::io::stdin();
        let mut deserializer = serde_json::Deserializer::from_reader(stdin);
        Message::deserialize(&mut deserializer)
    }

    fn send_message<T: Serialize>(&self, message: &Message<T>) -> serde_json::Result<()> {
        println!("{}", serde_json::to_string(message)?);
        Ok(())
    }
    fn send_body<T: Serialize>(&self, to: &str, body: &T) -> serde_json::Result<()> {
        let message = Message {
            src: self.node_id.clone(),
            dest: to.to_string(),
            body,
        };
        self.send_message(&message)
    }
    /// Respond to a message. If the message has a msg_id, set the in_reply_to appropriately
    fn respond<T: Serialize, U: Serialize>(
        &self,
        to: &Message<T>,
        body: &U,
    ) -> serde_json::Result<()> {
        let original_body_map = match to_value(&to.body)? {
            Value::Object(map) => map,
            _ => panic!("Message body is not an object"),
        };
        if let Some(msg_id) = original_body_map.get("msg_id") {
            let mut new_body_map = match to_value(body)? {
                Value::Object(map) => map,
                _ => panic!("Message body is not an object"),
            };
            new_body_map.insert("in_reply_to".to_string(), msg_id.clone());
            self.send_body(&to.src, &new_body_map)
        } else {
            self.send_body(&to.src, body)
        }
    }
}

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
