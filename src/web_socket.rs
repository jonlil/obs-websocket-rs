pub use crate::events::ObsEvent;
pub use crate::requests::{Request as ObsRequest, RequestType as ObsRequestType};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{mpsc::sync_channel, Arc, Mutex};
use tungstenite::{client::AutoStream, connect, Message, WebSocket};
use url::Url;

pub struct ObsWebSocket {
    socket: Arc<Mutex<WebSocket<AutoStream>>>,
    counter: usize,
}

pub type ObsMessageArguments = HashMap<String, Value>;
pub trait ObsEventEmitter {
    fn on_event(&self, event: ObsEvent);
}

#[derive(Deserialize, Debug)]
pub struct ObsResponse {
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

    pub fn read(&mut self) -> tungstenite::Result<Message> {
        self.socket.clone().lock().unwrap().read_message()
    }

    pub fn find_scenes(&mut self) -> Result<String, &'static str> {
        match self.send(ObsRequestType::GetSceneList, None) {
            Ok(message) => {
                eprintln!("{:?}", message);

                Ok("something".to_string())
            }
            Err(_) => Err("Failed finding scenes"),
        }
    }

    fn authenticate(&mut self, password: String) {
        let message = self.send(ObsRequestType::GetAuthRequired, None).unwrap();
        let response: ObsAuthRequiredResponse = serde_json::from_str(&message).unwrap();

        let hashed_secret = hash(response.salt, response.challenge, password);
        let mut auth_arguments = HashMap::new();
        auth_arguments.insert("auth".to_string(), Value::String(hashed_secret));

        self.send(ObsRequestType::Authenticate, Some(auth_arguments));
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
                    match serde_json::from_str(&event) {
                        Ok(message) => tx.on_event(message),
                        Err(err) => {
                            eprintln!("Failed parsing message: {:#?}, raw: {:?}", err, event)
                        }
                    };
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
        let message = ObsRequest::new(request_type, &message_id, args);
        let payload = serde_json::to_string(&message).unwrap();

        // Lock the socket since we're sending a request that we also
        // want to get the response for.
        match self.socket.clone().lock() {
            Ok(mut socket) => {
                // Send message to OBS
                socket.write_message(Message::Text(payload));
                match read(&mut socket) {
                    Ok((response, message)) => {
                        if &response.message_id == message_id {
                            Ok(message)
                        } else {
                            Err("Invalid message_id")
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            // TODO: Handle Mutex posion
            Err(_err) => Err("failed allocating socket lock"),
        }
    }
}

fn read(socket: &mut WebSocket<AutoStream>) -> Result<(ObsResponse, String), &'static str> {
    match socket.read_message() {
        Ok(message) => match serde_json::from_str(message.to_text().unwrap()) {
            Ok(body) => Ok((body, message.to_string())),
            Err(err) => {
                eprintln!("Error parsing JSON message: {:#?}, raw: {:?}", err, message);
                Err("Failed parsing JSON")
            }
        },
        Err(_err) => Err("Failed reading from socket"),
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
