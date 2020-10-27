use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "cmd")]
pub enum Cmd {
    Version { callback: String, error: String },
    GetAdapterInfo { callback: String, error: String },
}
