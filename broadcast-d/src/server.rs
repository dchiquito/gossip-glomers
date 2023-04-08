use std::{collections::hash_map::DefaultHasher, io::Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Result;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<T> {
    pub src: String,
    pub dest: String,
    pub body: Body<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<T> {
    pub msg_id: Option<u64>,
    pub in_reply_to: Option<u64>,
    #[serde(flatten)]
    pub fields: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
}

pub struct Server {}

impl Server {
    fn init() -> (Server, Message<InitPayload>) {
        let server = Server {};
        let init_message: Message<InitPayload> = server.read_message().unwrap();
        eprintln!("initin {:?}", init_message);
        (server, init_message)
    }
    pub fn read_message<T: DeserializeOwned>(&self) -> Result<Message<T>> {
        let stdin = std::io::stdin().lock();
        let mut deserializer = serde_json::Deserializer::from_reader(stdin);
        Message::deserialize(&mut deserializer)
    }
}

pub struct Sender {
    pub node_id: String,
    pub node_ids: Vec<String>,
    counter: u64,
}

impl Sender {
    fn init(init_message: &Message<InitPayload>) -> Result<Sender> {
        let mut sender = match &init_message.body.fields {
            InitPayload::Init { node_id, node_ids } => {
                // Calculate a unique starting counter index using the hash of the node ID
                let mut hasher = DefaultHasher::new();
                node_id.hash(&mut hasher);
                let counter = hasher.finish();
                Sender {
                    node_id: node_id.clone(),
                    node_ids: node_ids.clone(),
                    counter,
                }
            }
            _ => panic!("Invalid init message"),
        };
        let init_ok = InitPayload::InitOk {};
        eprintln!("blastin {:?}", init_ok);
        sender.respond(init_message, init_ok)?;
        Ok(sender)
    }
    /// Determine neighbors to ensure we can reach any other node in the network in two hops
    pub fn sane_neighbors(&self) -> Vec<String> {
        let mut neighbors = vec![];
        let my_index = self
            .node_ids
            .iter()
            .position(|n| n == &self.node_id)
            .expect("node_id was not in the node_ids list");
        let mut pow_two = 1;
        while pow_two < self.node_ids.len() / 2 {
            neighbors.push(self.node_ids[(my_index + pow_two) % self.node_ids.len()].to_string());
            pow_two *= 2;
        }
        neighbors
    }
    /// Write a message directly to stdout
    pub fn send_message<T: Serialize>(&self, message: &Message<T>) -> Result<()> {
        let stdout = std::io::stdout().lock();
        let mut serializer = serde_json::Serializer::new(stdout);
        message.serialize(&mut serializer)?;
        serializer
            .into_inner()
            .write_all(b"\n")
            .expect("Error writing newline");
        Ok(())
    }
    /// Adds the msg_id field to a body and wraps it in a Message
    pub fn message<T: Serialize>(&mut self, to: &str, fields: T) -> Result<Message<T>> {
        let msg_id = self.counter;
        self.counter += 1;
        let body = Body {
            msg_id: Some(msg_id),
            in_reply_to: None,
            fields,
        };
        Ok(Message {
            src: self.node_id.clone(),
            dest: to.to_string(),
            body,
        })
    }
    /// Creates a response to a message by setting the msg_id and in_reply_to fields
    pub fn response<T, U>(&mut self, to: &Message<T>, fields: U) -> Result<Message<U>>
    where
        T: Serialize,
        U: Serialize,
    {
        let mut message = self.message(&to.src, fields)?;
        message.body.in_reply_to = to.body.msg_id;
        Ok(message)
    }
    /// Send a message body to stdout
    pub fn send<T: Serialize>(&mut self, to: &str, fields: T) -> Result<()> {
        let message = self.message(to, fields)?;
        self.send_message(&message)
    }
    /// Respond to a message. If the message has a msg_id, set the in_reply_to appropriately
    pub fn respond<T: Serialize, U: Serialize>(
        &mut self,
        to: &Message<T>,
        fields: U,
    ) -> Result<()> {
        let message = self.response(to, fields)?;
        self.send_message(&message)
    }
}

pub fn init() -> Result<(Server, Sender)> {
    let (server, init_message) = Server::init();
    let sender = Sender::init(&init_message)?;
    Ok((server, sender))
}
