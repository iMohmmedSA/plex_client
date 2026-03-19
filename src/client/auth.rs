use serde::Serialize;

use crate::{
    client::{
        Base, Client,
        crypto::{DeviceKeypair, JwtClaims},
    },
    error::Result,
    models::auth::{
        pin::PinResponse,
        token::{NonceResponse, TokenResponse},
    },
};

#[derive(Debug, Clone, Serialize)]
struct Jwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
    pub kid: String,
    pub alg: String,
}

#[derive(Debug, Clone, Serialize)]
struct PinRequest {
    pub jwk: Jwk,
    pub strong: bool,
}

impl DeviceKeypair {
    fn to_jwk(&self) -> Jwk {
        Jwk {
            kty: "OKP".to_string(),
            crv: "Ed25519".to_string(),
            x: self.public_key_b64(),
            kid: self.kid.clone(),
            alg: "EdDSA".to_string(),
        }
    }
}

impl Client {
    pub async fn request_pin(&self, strong: bool, keypair: &DeviceKeypair) -> Result<PinResponse> {
        let body = PinRequest {
            jwk: keypair.to_jwk(),
            strong,
        };
        self.post(Base::Clients, "/api/v2/pins", Some(body)).await
    }

    pub fn auth_url(&self, pin: &PinResponse, forward_url: Option<&str>) -> String {
        let code = &pin.code;

        if code.len() == 4 {
            return format!("https://plex.tv/link/?pin={}", code);
        }

        let mut params = format!("clientID={}&code={}", self.inner.client_id, code,);

        if let Some(p) = &self.inner.product {
            params.push_str(&format!(
                "&{}={}",
                urlencoding::encode("context[device][product]"),
                urlencoding::encode(p)
            ));
        }

        if let Some(forward) = forward_url {
            params.push_str(&format!("&forwardUrl={}", urlencoding::encode(forward)));
        }

        format!("https://app.plex.tv/auth#?{}", params)
    }

    pub async fn poll_pin(&self, pin_id: u64, keypair: &DeviceKeypair) -> Result<PinResponse> {
        let now = chrono::Utc::now();
        let claims = JwtClaims {
            aud: "plex.tv".to_string(),
            iss: self.inner.client_id.clone(),
            iat: now.timestamp(),
            exp: (now + chrono::Duration::hours(1)).timestamp(),
            nonce: None,
            scope: None,
        };
        let signed_jwt = keypair.sign_jwt(&claims)?;
        let path = format!(
            "/api/v2/pins/{}?deviceJWT={}",
            pin_id,
            urlencoding::encode(&signed_jwt)
        );
        self.get(Base::Clients, &path).await
    }

    pub async fn refresh_token(&self, keypair: &DeviceKeypair, scope: &str) -> Result<String> {
        let nonce: NonceResponse = self.get(Base::Clients, "/api/v2/auth/nonce").await?;

        let now = chrono::Utc::now();
        let claims = JwtClaims {
            aud: "plex.tv".to_string(),
            iss: self.inner.client_id.clone(),
            iat: now.timestamp(),
            exp: (now + chrono::Duration::hours(1)).timestamp(),
            nonce: Some(nonce.nonce),
            scope: Some(scope.to_string()),
        };
        let signed_jwt = keypair.sign_jwt(&claims)?;

        let body = serde_json::json!({ "jwt": signed_jwt });
        let response: TokenResponse = self
            .post(Base::Clients, "/api/v2/auth/token", Some(body))
            .await?;

        Ok(response.auth_token)
    }
}
