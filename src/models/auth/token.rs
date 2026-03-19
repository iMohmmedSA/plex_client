use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub auth_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}
