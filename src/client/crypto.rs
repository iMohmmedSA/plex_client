use crate::error::Result;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{SigningKey, VerifyingKey, ed25519::signature::Signer};
use rand::random;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub(crate) struct JwtClaims {
    pub(crate) aud: String,
    pub(crate) iss: String,
    pub(crate) iat: i64,
    pub(crate) exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceKeypair {
    pub(crate) signing_key: SigningKey,
    pub(crate) kid: String,
}

impl DeviceKeypair {
    /// Generate a new random Ed25519 keypair
    pub fn generate() -> Self {
        // TODO: UPDATE ed25519_dalek to v3 then use the generation
        // let signing_key = SigningKey::generate(&mut RngCoro);
        let bytes: [u8; 32] = random();
        let signing_key = SigningKey::from_bytes(&bytes);
        let kid = Uuid::new_v4().to_string();
        Self { signing_key, kid }
    }

    /// Restore from saved bytes
    pub fn from_bytes(private_key_bytes: &[u8; 32], kid: &str) -> Self {
        let signing_key = SigningKey::from_bytes(private_key_bytes);
        Self {
            signing_key,
            kid: kid.to_string(),
        }
    }

    /// The bytes the user must persist (private key)
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// The key ID the user must also persist alongside private key bytes
    pub fn kid(&self) -> &str {
        &self.kid
    }

    pub(crate) fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Base64url encoded public key for JWK
    pub(crate) fn public_key_b64(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.verifying_key().as_bytes())
    }

    pub(crate) fn sign_jwt(&self, claims: &JwtClaims) -> Result<String> {
        // Build header
        let header = json!({
            "kid": self.kid,
            "alg": "EdDSA",
            "typ": "JWT"
        });

        let header_b64 = URL_SAFE_NO_PAD.encode(header.to_string().as_bytes());
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(claims)?.as_bytes());
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // Sign with private key
        let signature = self.signing_key.sign(signing_input.as_bytes());
        let signature_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        Ok(format!("{}.{}", signing_input, signature_b64))
    }
}
