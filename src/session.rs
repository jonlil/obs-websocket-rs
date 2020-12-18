use crate::requests::RequestType;
use crate::web_socket::{ObsEventEmitter, ObsMessageArguments, ObsWebSocket};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};

pub struct ObsSession(Arc<Mutex<ObsWebSocket>>);

impl ObsSession {
    pub fn new(obws: ObsWebSocket) -> Self {
        ObsSession(Arc::new(Mutex::new(obws)))
    }

    pub fn send<R>(
        &self,
        rt: RequestType,
        args: Option<ObsMessageArguments>,
    ) -> Result<R, serde_json::Error>
    where
        R: DeserializeOwned,
    {
        let response = self.0.clone().lock().unwrap().send(rt, args);

        serde_json::from_str(&response.unwrap())
    }

    pub fn run(
        &self,
        tx: Box<dyn ObsEventEmitter + Send + 'static>,
    ) -> std::thread::JoinHandle<String> {
        let listener = self.0.clone();
        std::thread::spawn(move || loop {
            match listener.lock().unwrap().read() {
                Ok(payload) => {
                    match serde_json::from_str(&payload.to_string()) {
                        Ok(message) => tx.on_event(message),
                        Err(err) => {
                            eprintln!("Failed parsing message: {:#?}, raw: {:?}", err, payload)
                        }
                    };
                }
                _ => {}
            };
        })
    }
}
