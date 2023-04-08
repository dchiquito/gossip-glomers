use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    sync::{Arc, Mutex},
    time::Instant,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_value, to_value, Map, Result, Value};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<T> {
    pub src: String,
    pub dest: String,
    pub body: T,
}

impl Message<Map<String, Value>> {
    pub fn cast<T: DeserializeOwned>(&self) -> Result<Message<T>> {
        let body = from_value(serde_json::Value::Object(self.body.clone()))?;
        Ok(Message {
            src: self.src.clone(),
            dest: self.src.clone(),
            body,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<T> {
    pub src: String,
    pub dest: String,
    pub body: T,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Body<T> {
//     pub msg_id: u64,
//     pub in_reply_to: u64,
//     #[serde(flatten)]
//     pub fields: T,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "type")]
// #[serde(rename_all = "snake_case")]
// pub enum Payload {
//     ...
// }

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

type Handler = dyn FnMut(&mut Sender, &Message<Map<String, Value>>) -> Result<()>;

pub struct Server {
    pub sender: Arc<Mutex<Sender>>,
    handlers: HashMap<String, Box<Handler>>,
}

impl Server {
    pub fn new() -> Server {
        let server = Server {
            sender: Arc::new(Mutex::new(Sender {
                node_id: "".to_string(),
                node_ids: vec![],
                counter: 0,
                // pending_rpcs: HashMap::new(),
            })),
            handlers: HashMap::new(),
        };
        let init: Message<Init> = server.read_message().unwrap();
        server.sender.lock().unwrap().init(&init);
        let init_ok = InitOk::new(&init.body);
        server
            .sender
            .lock()
            .unwrap()
            .respond(&init, &init_ok)
            .unwrap();
        server
    }
    pub fn read_message<T: DeserializeOwned>(&self) -> Result<Message<T>> {
        let stdin = std::io::stdin();
        let mut deserializer = serde_json::Deserializer::from_reader(stdin);
        Message::deserialize(&mut deserializer)
    }
    pub fn handle<F>(&mut self, r#type: &str, handler: F)
    where
        F: FnMut(&mut Sender, &Message<Map<String, Value>>) -> Result<()> + 'static,
    {
        self.handlers.insert(r#type.to_string(), Box::new(handler));
    }
    pub fn serve(&mut self) -> serde_json::Result<()> {
        loop {
            let message: Message<Map<String, Value>> = self.read_message()?;
            eprintln!("Handling {:?}", message);
            // Check if we are waiting for a response for that msg_id
            // if let Some(Value::Number(in_reply_to)) = message.body.get("in_reply_to") {
            //     eprintln!("gotta replyto {:?}", in_reply_to);
            //     if let Some((_, _, mut handler)) = self
            //         .sender
            //         .pending_rpcs
            //         .remove(&in_reply_to.as_u64().unwrap())
            //     {
            //         eprintln!("gotta rhandle ");
            //         handler(&mut self.sender, &message)?;
            //         continue;
            //     }
            // }
            // Check if there is a valid handler
            if let Some(handler) = self.handlers.get_mut(
                message
                    .body
                    .get("type")
                    .expect("Message had no type")
                    .as_str()
                    .expect("Message type was not a string"),
            ) {
                handler(&mut self.sender.lock().unwrap(), &message)?;
                continue;
            }
            panic!("No handler for {:?}", message);
        }
    }
}

// type PendingMessage = (Instant, Message<Map<String, Value>>, Box<Handler>);

pub struct Sender {
    pub node_id: String,
    pub node_ids: Vec<String>,
    counter: u64,
    // pending_rpcs: HashMap<u64, PendingMessage>,
}

impl Sender {
    fn init(&mut self, init: &Message<Init>) {
        self.node_id = init.body.node_id.clone();
        self.node_ids = init.body.node_ids.clone();
        let mut hasher = DefaultHasher::new();
        self.node_id.hash(&mut hasher);
        self.counter = hasher.finish();
    }
    /// Adds the msg_id field to a body and wraps it in a Message
    pub fn message<T: Serialize>(
        &mut self,
        to: &str,
        body: T,
    ) -> Result<Message<Map<String, Value>>> {
        let mut body_map = match to_value(body)? {
            Value::Object(map) => map,
            _ => panic!("Message body is not an object"),
        };
        let msg_id = self.counter;
        self.counter += 1;
        body_map.insert("msg_id".to_string(), to_value(msg_id)?);
        Ok(Message {
            src: self.node_id.clone(),
            dest: to.to_string(),
            body: body_map,
        })
    }
    /// Write a message to stdout. msg_id will not be included
    pub fn send_message<T: Serialize>(&self, message: &Message<T>) -> Result<()> {
        println!("{}", serde_json::to_string(message)?);
        Ok(())
    }
    /// Send a message body to stdout
    pub fn send_body<T: Serialize>(&mut self, to: &str, body: T) -> Result<()> {
        let message = self.message(to, body)?;
        self.send_message(&message)
    }
    /// Respond to a message. If the message has a msg_id, set the in_reply_to appropriately
    pub fn respond<T: Serialize, U: Serialize>(&mut self, to: &Message<T>, body: &U) -> Result<()> {
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
            eprintln!("RESPONSIN {:?}", new_body_map);
            self.send_body(&to.src, &new_body_map)
        } else {
            self.send_body(&to.src, body)
        }
    }
    // TODO rpc callbacks are not thread safe
    // pub fn rpc<T: Serialize, F>(&mut self, to: &str, body: &T, callback: F) -> Result<()>
    // where
    //     F: FnMut(&mut Sender, &Message<Map<String, Value>>) -> Result<()> + 'static,
    // {
    //     let message = self.message(to, body)?;
    //     self.pending_rpcs.insert(
    //         message.body.get("msg_id").unwrap().as_u64().unwrap(),
    //         (Instant::now(), message.clone(), Box::new(callback)),
    //     );
    //     self.send_message(&message)
    // }

    // TODO retry unacked messages?
    // pub fn check_for_timeouts() {}
}
