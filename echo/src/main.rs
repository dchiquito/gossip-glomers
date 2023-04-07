use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_value, to_value, Map, Value};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Map<String, Value>,
}

impl Message {
    fn parse_body<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        from_value(Value::Object(self.body.clone()))
    }
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

#[derive(Serialize, Deserialize, Debug)]
struct Echo {
    r#type: String,
    msg_id: u32,
    in_reply_to: Option<u32>,
    echo: Option<String>,
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
        let message = server.read_message().unwrap();
        let init: Init = message.parse_body().unwrap();
        let init_ok = InitOk::new(&init);
        server.node_id = init.node_id;
        server.node_ids = init.node_ids;
        server.respond(message, init_ok);
        server
    }
    fn read_message(&self) -> serde_json::Result<Message> {
        let stdin = std::io::stdin();
        let mut deserializer = serde_json::Deserializer::from_reader(stdin);
        Message::deserialize(&mut deserializer)
    }

    fn write_message<T: Serialize>(&self, to: &str, body: T) -> serde_json::Result<()> {
        let message = match to_value(body)? {
            Value::Object(body_map) => Message {
                src: self.node_id.clone(),
                dest: to.to_string(),
                body: body_map,
            },
            _ => panic!("Message body is not an object"),
        };
        println!("{}", serde_json::to_string(&message)?);
        Ok(())
    }
    fn respond<T: Serialize>(&self, to: Message, body: T) -> serde_json::Result<()> {
        self.write_message(&to.src, body)
    }
}

fn main() -> serde_json::Result<()> {
    let server = Server::new();
    loop {
        let message = server.read_message()?;
        let mut echo: Echo = message.parse_body()?;
        echo.r#type = "echo_ok".to_string();
        echo.in_reply_to = Some(echo.msg_id);
        server.respond(message, echo);
    }
}
