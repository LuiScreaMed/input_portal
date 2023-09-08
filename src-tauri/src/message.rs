use serde::{Serialize, Deserialize};

use crate::{config::Config, inputs::InputMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Input,
    Config,
    Test
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub r#type: MessageType,
    pub data: MessageData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageData {
    ConfigMessage(Config),
    InputMessage(InputMessage),
    TestMessage(String),
}