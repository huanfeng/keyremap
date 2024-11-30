use std::fmt;

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

impl fmt::Display for InputEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref key_value) = self.key {
            write!(f, "key: {:?}", key_value)
        } else if let Some(ref button_value) = self.button {
            write!(f, "button: {:?}", button_value)
        } else {
            write!(f, "none")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OutputEvent {
    #[serde(default)]
    pub key: Option<Key>,
    #[serde(default)]
    pub combination: Option<Vec<KeyDef>>,
}

impl fmt::Display for OutputEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref combination_value) = self.combination {
            let string = combination_value
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            write!(f, "combination: [{}]", string)
        } else if let Some(ref key_value) = self.key {
            write!(f, "key: {:?}", key_value)
        } else {
            write!(f, "none")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KeyDef {
    pub key: Key,
}

impl fmt::Display for KeyDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "key: {:?}", self.key)
    }
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
