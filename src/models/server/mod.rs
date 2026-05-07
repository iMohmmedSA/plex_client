use serde::Deserialize;
pub mod identity;

pub use identity::Identity;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaContainer<T> {
    media_container: T,
}
