use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tungstenite::{client::AutoStream, connect, Message, WebSocket};
use url::Url;

pub type ObsMessageArguments = HashMap<String, Value>;

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
    socket: WebSocket<AutoStream>,
    counter: usize,
}

impl ObsWebSocket {
    pub fn connect(url: &str, password: String) -> Self {
        let connect_url = Url::parse(url).expect("Failed connecting");
        let (socket, _response) = connect(connect_url).unwrap();
        let mut obs = ObsWebSocket { socket, counter: 0 };

        obs.authenticate(password);

        obs
    }

    pub fn read(&mut self) -> tungstenite::Result<Message> {
        self.socket.read_message()
    }

    fn authenticate(&mut self, password: String) {
        let message = self.send(ObsRequestType::GetAuthRequired, None).unwrap();
        let response: ObsAuthRequiredResponse = serde_json::from_str(&message).unwrap();

        let hashed_secret = hash(response.salt, response.challenge, password);
        let mut auth_arguments = HashMap::new();
        auth_arguments.insert("auth".to_string(), Value::String(hashed_secret));
        let message = self.send(ObsRequestType::Authenticate, Some(auth_arguments));
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
        self.socket.write_message(Message::Text(payload));

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

    #[test]
    fn it_connects() {
        let obs = ObsWebSocket::connect("ws://192.168.10.106:4444", "test-password".to_string());

        assert_eq!(true, false);
    }
}
