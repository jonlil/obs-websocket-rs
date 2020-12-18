use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
#[serde(tag = "request-type")]
pub enum RequestType {
    GetAuthRequired,
    Authenticate,
    GetSceneList,
    SetCurrentScene,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Request<'a> {
    #[serde(flatten)]
    request_type: RequestType,

    message_id: &'a str,

    #[serde(flatten)]
    args: Option<HashMap<String, Value>>,
}

impl<'a> Request<'a> {
    pub fn new(
        request_type: RequestType,
        message_id: &'a str,
        args: Option<HashMap<String, Value>>,
    ) -> Self {
        Request {
            request_type,
            message_id,
            args,
        }
    }
}
