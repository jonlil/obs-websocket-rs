use serde::Deserialize;

use crate::events::ObsSource;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Scene {
    pub name: String,
    pub sources: Vec<ObsSource>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct GetSceneList {
    message_id: String,
    pub current_scene: String,
    pub scenes: Vec<Scene>,
}
