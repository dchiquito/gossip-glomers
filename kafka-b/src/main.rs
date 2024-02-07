use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use server::Message;

mod server;

type Entry = usize;
type Offset = i64;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum P {
    Send {
        key: String,
        msg: Entry,
    },
    SendOk {
        offset: Offset,
    },
    Poll {
        offsets: HashMap<String, Offset>,
    },
    PollOk {
        msgs: HashMap<String, Vec<(Offset, Entry)>>,
    },
    CommitOffsets {
        offsets: HashMap<String, Offset>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, Offset>,
    },
    Error {
        code: u64,
        text: String,
    },
}

fn binary_search<T>(arr: &[(Offset, T)], offset: Offset) -> usize {
    if arr[0].0 >= offset {
        return 0;
    }
    let mut left = 0;
    let mut right = arr.len();
    while left + 1 < right {
        let index = left + ((right - left) / 2);
        if arr[index].0 < offset {
            left = index;
        } else {
            right = index;
        }
    }
    right
}

fn main() -> serde_json::Result<()> {
    let (server, mut sender) = server::init()?;
    let mut logs = HashMap::<String, Vec<(Offset, Entry)>>::new();
    let mut commits = HashMap::<String, Offset>::new();
    loop {
        let message: Message<P> = server.read_message()?;
        match &message.body.fields {
            P::Send { key, msg } => {
                if !logs.contains_key(key) {
                    logs.insert(key.clone(), vec![]);
                }
                let log = &mut logs.get_mut(key).unwrap();
                let offset = chrono::offset::Utc::now().timestamp_micros();
                log.push((offset, *msg));
                sender.respond(&message, &P::SendOk { offset })?;
            }
            P::Poll { offsets } => {
                let msgs = offsets
                    .iter()
                    .filter(|(key, _)| logs.contains_key(*key))
                    .map(|(key, &offset)| {
                        let log = logs.get(key).unwrap();
                        let index = binary_search(log, offset);
                        (
                            key.clone(),
                            // Just in case, limit response to 10 entries
                            Vec::from(&log[index..log.len().min(index + 10)]),
                        )
                    })
                    .collect();
                sender.respond(&message, &P::PollOk { msgs })?;
            }
            P::CommitOffsets { offsets } => {
                offsets.iter().for_each(|(key, &offset)| {
                    commits.insert(key.clone(), offset);
                });
                sender.respond(&message, &P::CommitOffsetsOk {})?;
            }
            P::ListCommittedOffsets { keys } => {
                let offsets = keys
                    .iter()
                    .filter(|key| commits.contains_key(*key))
                    .map(|key| (key.clone(), *commits.get(key).unwrap()))
                    .collect();
                sender.respond(&message, &P::ListCommittedOffsetsOk { offsets })?;
            }
            _ => panic!("NOT ALLOWED: {:?}", message),
        }
    }
}
