use serde::Deserialize;
pub mod identity;

pub use identity::Identity;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer<T> {
    #[serde(rename = "MediaContainer")]
    pub media_container: T,
}
