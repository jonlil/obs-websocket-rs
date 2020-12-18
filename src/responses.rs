use serde::Deserialize;

use crate::events::ObsSource;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Scene {
    name: String,
    sources: Vec<ObsSource>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GetSceneList {
    message_id: String,
    current_scene: String,
    scenes: Vec<Scene>,
}
