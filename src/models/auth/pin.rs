use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PinResponse {
    pub id: u64,
    pub code: String,
    pub product: String,
    pub trusted: bool,
    pub qr: String,
    pub client_identifier: String,
    pub location: PinLocation,
    pub expires_in: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub auth_token: Option<String>,
    pub new_registration: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PinLocation {
    pub code: String,
    pub european_union_member: bool,
    pub continent_code: String,
    pub country: String,
    pub city: String,
    pub time_zone: String,
    pub postal_code: String,
    pub in_privacy_restricted_country: bool,
    pub in_privacy_restricted_region: bool,
    pub subdivisions: String,
    pub coordinates: String,
}
