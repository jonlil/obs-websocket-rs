#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

mod events;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{mpsc::sync_channel, Arc, Mutex};
use tungstenite::{client::AutoStream, connect, Message, WebSocket};
use url::Url;

pub use events::ObsEvent;

pub type ObsMessageArguments = HashMap<String, Value>;
pub trait ObsEventEmitter {
    fn on_event(&self, event: ObsEvent);
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct ObsRequest<'a> {
    #[serde(flatten)]
    request_type: ObsRequestType,

    message_id: &'a str,

    #[serde(flatten)]
    args: Option<HashMap<String, Value>>,
}

#[derive(Deserialize, Debug)]
struct ObsResponse {
    #[serde(rename(deserialize = "message-id"))]
    message_id: String,
}

#[derive(Deserialize, Debug)]
struct ObsAuthRequiredResponse {
    #[serde(rename(deserialize = "authRequired"))]
    auth_required: bool,

    challenge: String,

    #[serde(rename(deserialize = "message-id"))]
    message_id: String,

    salt: String,
    status: String,
}

#[derive(Serialize)]
#[serde(tag = "request-type")]
pub enum ObsRequestType {
    GetAuthRequired,
    Authenticate,
}

pub struct ObsWebSocket {
    socket: Arc<Mutex<WebSocket<AutoStream>>>,
    counter: usize,
}

impl ObsWebSocket {
    pub fn connect(url: &str, password: String) -> Self {
        let connect_url = Url::parse(url).expect("Failed connecting");
        let (socket, _response) = connect(connect_url).unwrap();
        let mut obs = ObsWebSocket {
            socket: Arc::new(Mutex::new(socket)),
            counter: 0,
        };

        obs.authenticate(password);

        obs
    }

    fn read(&mut self) -> tungstenite::Result<Message> {
        self.socket.clone().lock().unwrap().read_message()
    }

    fn authenticate(&mut self, password: String) {
        let message = self.send(ObsRequestType::GetAuthRequired, None).unwrap();
        let response: ObsAuthRequiredResponse = serde_json::from_str(&message).unwrap();

        let hashed_secret = hash(response.salt, response.challenge, password);
        let mut auth_arguments = HashMap::new();
        auth_arguments.insert("auth".to_string(), Value::String(hashed_secret));
        let message = self.send(ObsRequestType::Authenticate, Some(auth_arguments));
    }

    pub fn run<T>(&mut self, tx: T)
    where
        T: ObsEventEmitter,
    {
        let socket = self.socket.clone();
        let (sender, receiver) = sync_channel::<String>(1);
        std::thread::spawn(move || loop {
            if let Ok(message) = socket.lock().unwrap().read_message() {
                sender.send(message.to_string()).unwrap();
            }
        });

        loop {
            match receiver.recv() {
                Ok(event) => {
                    let event: ObsEvent = serde_json::from_str(&event).unwrap();
                    tx.on_event(event);
                }
                Err(_) => {}
            };
        }
    }

    fn message_id(&mut self) -> String {
        if self.counter >= usize::MAX {
            self.counter = 0;
        }

        self.counter += 1;
        self.counter.to_string()
    }

    pub fn send(
        &mut self,
        request_type: ObsRequestType,
        args: Option<ObsMessageArguments>,
    ) -> Result<String, &'static str> {
        let message_id = &self.message_id();
        let message = ObsRequest {
            message_id: &message_id,
            request_type,
            args,
        };
        let payload = serde_json::to_string(&message).unwrap();
        self.socket
            .clone()
            .lock()
            .unwrap()
            .write_message(Message::Text(payload));

        let message = self.read().unwrap();
        let response: ObsResponse = serde_json::from_str(message.to_text().unwrap()).unwrap();

        if &response.message_id == message_id {
            Ok(message.to_string())
        } else {
            Err("Invalid message_id")
        }
    }
}

fn hash(salt: String, challenge: String, password: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&password);
    hasher.update(&salt);

    let mut result = Sha256::new();
    result.update(&base64::encode(hasher.finalize()));
    result.update(&challenge);

    base64::encode(result.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_obs_message() {
        let mut args = HashMap::new();
        args.insert(
            "scene-name".to_string(),
            Value::String("pre-show".to_string()),
        );

        assert_eq!(
            serde_json::to_string(&ObsRequest {
                message_id: "12345",
                request_type: ObsRequestType::GetAuthRequired,
                args: Some(args),
            })
            .unwrap(),
            "{\"request-type\":\"GetAuthRequired\",\"message-id\":\"12345\",\"scene-name\":\"pre-show\"}"
        );
    }
}
