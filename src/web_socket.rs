pub use crate::events::ObsEvent;
pub use crate::requests::{Request as ObsRequest, RequestType as ObsRequestType};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{
    mpsc::{sync_channel, SyncSender},
    Arc, Mutex,
};
use tungstenite::{Message, WebSocket};
use url::Url;

pub trait ObsEventEmitter {
    fn on_event(&self, event: ObsEvent);
}
type ObsEventListeners = Arc<Mutex<HashMap<String, SyncSender<String>>>>;

pub struct ObsWebSocket {
    socket: Arc<Mutex<WebSocket<TcpStream>>>,
    counter: usize,
    listeners: ObsEventListeners,
}

pub type ObsMessageArguments = HashMap<String, Value>;

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
    fn client(url: &str) -> Result<WebSocket<TcpStream>, &'static str> {
        let connect_url = Url::parse(url).expect("Failed connecting");
        let addrs = connect_url.socket_addrs(|| Some(80)).unwrap();
        let stream = std::net::TcpStream::connect(&*addrs)
            .map_err(|_| "Failed connecting to WebSocket server")?;

        tungstenite::client::client(connect_url, stream)
            .map(|(socket, _response)| socket)
            .map_err(|_| "")
    }

    pub fn connect<T>(url: &str, password: String, tx: Box<T>) -> Result<Self, &'static str>
    where
        T: ObsEventEmitter + Send + 'static,
    {
        let mut obs = ObsWebSocket {
            socket: Arc::new(Mutex::new(Self::client(url)?)),
            counter: 0,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        };

        obs.run(tx);
        obs.authenticate(password);

        Ok(obs)
    }

    fn authenticate(&mut self, password: String) {
        let response: ObsAuthRequiredResponse =
            self.send(ObsRequestType::GetAuthRequired, None).unwrap();

        let hashed_secret = hash(response.salt, response.challenge, password);
        let mut auth_arguments = HashMap::new();
        auth_arguments.insert("auth".to_string(), Value::String(hashed_secret));

        self.send::<ObsResponse>(ObsRequestType::Authenticate, Some(auth_arguments));
    }

    pub fn run(&mut self, tx: Box<dyn ObsEventEmitter + Send + 'static>) {
        let socket = self.socket.clone();

        // Set the stream in non-blocking mode
        match socket.lock() {
            Ok(mut socket) => socket.get_mut().set_nonblocking(true).unwrap(),
            _ => panic!("Could not set socket in non-blocking mode"),
        };

        let listeners = self.listeners.clone();
        std::thread::spawn(move || loop {
            match socket.lock() {
                Ok(mut socket) => {
                    match socket.read_message() {
                        Ok(message) => process_event(message.to_string(), &listeners, &tx),
                        _ => {}
                    };
                }
                Err(err) => eprintln!("{:?}", err),
            };

            std::thread::sleep(std::time::Duration::from_millis(50));
        });
    }

    fn message_id(&mut self) -> String {
        if self.counter >= usize::MAX {
            self.counter = 0;
        }

        self.counter += 1;
        self.counter.to_string()
    }

    pub fn send<R>(
        &mut self,
        request_type: ObsRequestType,
        args: Option<ObsMessageArguments>,
    ) -> Result<R, &'static str>
    where
        R: DeserializeOwned,
    {
        let message_id = &self.message_id();
        let message = ObsRequest::new(request_type, &message_id, args);
        let payload = serde_json::to_string(&message).unwrap();
        let (sender, rx) = sync_channel::<String>(1);

        self.add_callback(message_id.into(), sender);

        match self.socket.clone().lock() {
            Ok(mut socket) => {
                // Send message to OBS
                socket
                    .write_message(Message::Text(payload))
                    .map_err(|_| "Failed sending websocket message to server")?;
            }
            // TODO: Handle Mutex posion
            Err(_err) => {}
        };

        match rx.recv() {
            Ok(message) => {
                serde_json::from_str(&message).map_err(|_err| "Failed parsing OBS Message")
            }
            Err(_err) => Err("Failed reading OBS response"),
        }
    }

    fn add_callback(&mut self, msg_id: String, sender: SyncSender<String>) {
        match self.listeners.lock() {
            Ok(mut listeners) => listeners.insert(msg_id, sender),
            Err(_err) => panic!("Failed adding callback listener"),
        };
    }
}

fn process_event(
    event: String,
    listeners: &ObsEventListeners,
    tx: &Box<dyn ObsEventEmitter + Send + 'static>,
) {
    if let Ok(message) = serde_json::from_str::<ObsResponse>(&event) {
        find_callback_listener(&listeners, &message.message_id).map(|tx| {
            match tx.send(event) {
                _ => {}
            };
        });
    } else {
        match serde_json::from_str(&event) {
            Ok(obs_event) => tx.on_event(obs_event),
            Err(_err) => {}
        };
    }
}

fn find_callback_listener(
    listeners: &ObsEventListeners,
    message_id: &str,
) -> Option<SyncSender<String>> {
    match listeners.lock() {
        Ok(mut listeners) => {
            if let Some(tx) = listeners.remove(message_id) {
                Some(tx)
            } else {
                None
            }
        }
        Err(_) => None,
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
