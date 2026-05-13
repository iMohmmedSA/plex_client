use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    size: u8,
    claimed: bool,
    machine_identifier: String,
    version: String,
}
