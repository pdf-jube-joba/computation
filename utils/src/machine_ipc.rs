use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Request {
    Model { name: String },
    Create { code: String, ainput: String },
    Step { rinput: String },
    Current,
    Drop,
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Response {
    Ok {
        message: Option<String>,
        model: Option<String>,
        create: Option<String>,
        step: Option<String>,
        snapshot: Option<String>,
    },
    Error {
        error: String,
    },
}

impl Response {
    pub fn ok_message(msg: impl Into<String>) -> Self {
        Self::Ok {
            message: Some(msg.into()),
            model: None,
            create: None,
            step: None,
            snapshot: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self::Error { error: msg.into() }
    }
}
