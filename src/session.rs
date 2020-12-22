use crate::requests::RequestType;
use crate::web_socket::{ObsMessageArguments, ObsWebSocket};
use serde::de::DeserializeOwned;

pub struct ObsSession(ObsWebSocket);

impl ObsSession {
    pub fn new(obws: ObsWebSocket) -> Self {
        ObsSession(obws)
    }

    pub fn send<R>(
        &mut self,
        rt: RequestType,
        args: Option<ObsMessageArguments>,
    ) -> Result<R, &'static str>
    where
        R: DeserializeOwned,
    {
        self.0.send(rt, args)
    }
}
