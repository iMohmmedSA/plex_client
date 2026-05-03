pub mod client;
pub mod endpoints;
pub mod error;
pub mod models;

pub(crate) mod headers {
    pub const CLIENT_ID: &str = "X-Plex-Client-Identifier";
    pub const TOKEN: &str = "X-Plex-Token";
    pub const PRODUCT: &str = "X-Plex-Product";
    pub const VERSION: &str = "X-Plex-Version";
    pub const PLATFORM: &str = "X-Plex-Platform";
    pub const PLATFORM_VERSION: &str = "X-Plex-Platform-Version";
    pub const DEVICE: &str = "X-Plex-Device";
    pub const DEVICE_VENDOR: &str = "X-Plex-Device-Vendor";
    pub const DEVICE_NAME: &str = "X-Plex-Device-Name";
    pub const MODEL: &str = "X-Plex-Model";
    pub const MARKETPLACE: &str = "X-Plex-Marketplace";
}
