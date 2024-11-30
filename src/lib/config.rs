use rdev::{Button, Key};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub version: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub comment: String,
    pub key_mappings: Vec<KeyMapping>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct KeyMapping {
    pub name: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default = "default_true")]
    pub enable: bool,
    pub from: InputEvent,
    pub to: OutputEvent,
}

#[derive(Debug, Deserialize)]
pub struct InputEvent {
    #[serde(default)]
    pub key: Option<Key>,
    #[serde(default)]
    pub button: Option<Button>,
}

#[derive(Debug, Deserialize)]
pub struct OutputEvent {
    #[serde(default)]
    pub key: Option<Key>,
    #[serde(default)]
    pub combination: Option<Vec<KeyDef>>,
}

#[derive(Debug, Deserialize)]
pub struct KeyDef {
    pub key: Key,
}

impl InputEvent {
    pub fn matches_key(&self, key: Key) -> bool {
        if let Some(ref key_value) = self.key {
            key_value == &key
        } else {
            false
        }
    }

    pub fn matches_button(&self, button: Button) -> bool {
        if let Some(ref button_value) = self.button {
            button_value == &button
        } else {
            false
        }
    }
}

impl OutputEvent {
    pub fn get_combination(&self) -> Option<Vec<Key>> {
        self.combination.as_ref().map(|combo| {
            combo
                .iter()
                .filter_map(|key_def| Some(key_def.key))
                .collect()
        })
    }
}
