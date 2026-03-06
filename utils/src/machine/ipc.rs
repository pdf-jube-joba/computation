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
    Ok { body: OkBody },
    Error { error: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum OkBody {
    Pong,
    Dropped,
    ModelSelected {
        model: String,
    },
    Created {
        model: String,
        create: String,
        snapshot: String,
    },
    Stepped {
        model: String,
        step: String,
        snapshot: String,
    },
    Current {
        model: String,
        snapshot: String,
    },
}

impl Response {
    pub fn ok(body: OkBody) -> Self {
        Self::Ok { body }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self::Error { error: msg.into() }
    }
}
