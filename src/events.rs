use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq)]
pub struct ObsSource {
    alignment: u32,
    cx: f32,
    cy: f32,
    id: u32,
    name: String,
    locked: bool,
    muted: bool,
    render: bool,
    source_cx: u32,
    source_cy: u32,

    #[serde(rename = "type")]
    kind: String,

    volume: f64,
    x: f32,
    y: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SceneItem {
    item_id: u32,
    source_name: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "update-type")]
pub enum ObsEvent {
    #[serde(rename_all = "kebab-case")]
    SwitchScenes {
        scene_name: String,
        sources: Vec<ObsSource>,
    },

    #[serde(rename_all = "kebab-case")]
    SceneItemSelected {
        item_id: u32,
        item_name: String,
        scene_name: String,
    },

    #[serde(rename_all = "kebab-case")]
    SceneItemDeselected {
        item_id: u32,
        item_name: String,
        scene_name: String,
    },

    #[serde(rename_all = "kebab-case")]
    SceneItemVisibilityChanged {
        item_id: u32,
        item_name: String,
        item_visible: bool,
        scene_name: String,
    },

    #[serde(rename_all = "kebab-case")]
    SceneItemLockChanged {
        item_id: u32,
        item_name: String,
        item_locked: bool,
        scene_name: String,
    },

    #[serde(rename_all = "kebab-case")]
    TransitionBegin {
        duration: u32,
        from_scene: String,
        to_scene: String,
        name: String,
        #[serde(rename = "type")]
        kind: String,
    },

    #[serde(rename_all = "kebab-case")]
    TransitionEnd {
        duration: u32,
        to_scene: String,
        name: String,
        #[serde(rename = "type")]
        kind: String,
    },

    #[serde(rename_all = "kebab-case")]
    TransitionVideoEnd {
        duration: u32,
        from_scene: String,
        to_scene: String,
        name: String,
        #[serde(rename = "type")]
        kind: String,
    },

    ScenesChanged,

    #[serde(rename_all = "camelCase")]
    SourceVolumeChanged {
        source_name: String,
        volume: f64,
    },

    #[serde(rename_all = "camelCase")]
    SourceMuteStateChanged {
        source_name: String,
        muted: bool,
    },

    #[serde(rename_all = "camelCase")]
    SourceDestroyed {
        source_kind: String,
        source_name: String,
        source_type: String,
    },

    #[serde(rename_all = "camelCase")]
    SourceCreated {
        source_name: String,
        source_type: String,
        source_settings: HashMap<String, String>,
    },

    #[serde(rename_all = "camelCase")]
    SourceFilterRemoved {
        filter_name: String,
        filter_type: String,
        source_name: String,
    },

    #[serde(rename_all = "kebab-case")]
    SourceOrderChanged {
        scene_name: String,
        scene_items: Vec<SceneItem>,
    },

    #[serde(rename_all = "kebab-case")]
    PreviewSceneChanged {
        scene_name: String,
        sources: Vec<ObsSource>,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_serialize_events() {
        let json: ObsEvent = serde_json::from_str(
"{\n    \"scene-name\": \"Game\",\n    \"sources\": [\n
            {
                \"alignment\": 5,\"cx\": 0.0,\"cy\": 0.0,\"id\": 7,
                \"locked\": false,
                \"muted\": false,
                \"name\": \"DJ\",
                \"render\": true,
                \"source_cx\": 0,
                \"source_cy\": 0,
                \"type\": \"coreaudio_input_capture\",
                \"volume\": 0.95903128385543823,
                \"x\": 0.0,
                \"y\": 0.0
            },
            {
                \"alignment\": 5,
                \"cx\": 525.0,
                \"cy\": 548.0,
                \"id\": 3,
                \"locked\": false,
                \"muted\": false,
                \"name\": \"Nacka Loga\",
                \"render\": true,
                \"source_cx\": 525,
                \"source_cy\": 548,
                \"type\": \"image_source\",
                \"volume\": 1.0,
                \"x\": 1148.0,
                \"y\": 14.0
            },\n        {\n            \"alignment\": 5,\n            \"cx\": 0.0,\n            \"cy\": 0.0,\n            \"id\": 2,\n            \"locked\": false,\n            \"muted\": false,\n            \"name\": \"Ishallsljud\",\n            \"render\": true,\n            \"source_cx\": 0,\n            \"source_cy\": 0,\n            \"type\": \"coreaudio_input_capture\",\n            \"volume\": 0.12957817316055298,\n            \"x\": 0.0,\n            \"y\": 0.0\n        },\n        {\n            \"alignment\": 5,\n            \"cx\": 0.0,\n            \"cy\": 0.0,\n            \"id\": 1,\n            \"locked\": false,\n            \"muted\": false,\n            \"name\": \"Logitech\",\n            \"render\": true,\n            \"source_cx\": 0,\n            \"source_cy\": 0,\n            \"type\": \"av_capture_input\",\n            \"volume\": 1.0,\n            \"x\": 0.0,\n            \"y\": 0.0\n        }\n    ],\n    \"update-type\": \"PreviewSceneChanged\"\n 
        }",
        )
        .unwrap();

        assert_eq!(
            json,
            ObsEvent::PreviewSceneChanged {
                scene_name: "Game",
                sources: vec![
                    ObsSource {
                        id: 7,
                        alignment: 5,
                        cx: 0.0,
                        cy: 0.0,
                        name: "DJ",
                        locked: false,
                        muted: false,
                        render: true,
                        source_cx: 0,
                        source_cy: 0,
                        kind: "coreaudio_input_capture",
                        volume: 0.95903128385543823,
                        x: 0.0,
                        y: 0.0,
                    },
                    ObsSource {
                        id: 3,
                        alignment: 5,
                        cx: 525.0,
                        cy: 548.0,
                        name: "Nacka Loga",
                        locked: false,
                        muted: false,
                        render: true,
                        source_cx: 525,
                        source_cy: 548,
                        kind: "image_source",
                        volume: 1.0,
                        x: 1148.0,
                        y: 14.0,
                    },
                    ObsSource {
                        id: 2,
                        alignment: 5,
                        cx: 0.0,
                        cy: 0.0,
                        name: "Ishallsljud",
                        locked: false,
                        muted: false,
                        render: true,
                        source_cx: 0,
                        source_cy: 0,
                        kind: "coreaudio_input_capture",
                        volume: 0.12957817316055298,
                        x: 0.0,
                        y: 0.0,
                    },
                    ObsSource {
                        id: 1,
                        alignment: 5,
                        cx: 0.0,
                        cy: 0.0,
                        name: "Logitech",
                        locked: false,
                        muted: false,
                        render: true,
                        source_cx: 0,
                        source_cy: 0,
                        kind: "av_capture_input",
                        volume: 1.0,
                        x: 0.0,
                        y: 0.0,
                    },
                ]
            }
        );
    }
}
