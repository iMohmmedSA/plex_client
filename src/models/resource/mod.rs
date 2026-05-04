use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub protocol: String,
    pub address: String,
    pub port: u16,
    pub uri: String,
    pub local: bool,
    pub relay: bool,
    #[serde(rename = "IPv6")]
    pub ipv6: bool,
}

// TODO: I am not sure of what is optional
// expected to break
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub name: String,
    pub product: String,
    pub product_version: String,
    pub platform: String,
    pub platform_version: String,
    pub device: Option<String>,
    pub client_identifier: String,
    pub provides: String,
    pub owner_id: Option<u32>,
    pub source_title: Option<String>,
    pub public_address: String,
    // access_token would be missing for Phones
    pub access_token: Option<String>,
    pub search_enabled: bool,
    pub created_at: String,
    pub last_seen_at: String,
    pub owned: bool,
    pub home: bool,
    pub synced: bool,
    pub relay: bool,
    pub presence: bool,
    pub https_required: bool,
    pub public_address_matches: bool,
    pub dns_rebinding_protection: Option<bool>,
    pub nat_loopback_supported: Option<bool>,
    pub connections: Vec<Connection>,
}
