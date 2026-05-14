use std::fs;

use plex_client::client::{Client, crypto::DeviceKeypair};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let login = SavedLogin::load();
    let client = Client::builder()
        .client_identifier(&login.client_id)
        .token(&login.token)
        .build()
        .unwrap();

    let keypair = DeviceKeypair::from_bytes(&login.private_key, &login.kid);

    let token = client
        .refresh_token(&keypair, "username,email,friendly_name")
        .await
        .unwrap_or_else(|e| panic!("error at: {e}"));

    login.save(&token);
}

#[derive(Serialize, Deserialize)]
struct SavedLogin {
    token: String,
    client_id: String,

    // Keypair
    kid: String,
    private_key: [u8; 32],
}

impl SavedLogin {
    fn load() -> Self {
        let s = fs::read_to_string("login.json")
            .expect("You are missing login.json. run auth.rs first");
        serde_json::from_str(&s).expect("Delete login.json then run auth again.")
    }

    fn save(mut self, token: &str) {
        self.token = token.to_string();
        let json = serde_json::to_string_pretty(&self).expect("Failed to serialize");
        fs::write("login.json", json).expect("Failed to write login.json");

        println!("\n\n========================================================");
        println!("\nYou are ready to run the other examples.");
        println!("Your token is saved at login.json.");
        println!("Once you are done make sure to remove the access");
        println!("Visit https://app.plex.tv/desktop/#!/settings/devices/all");
    }
}
