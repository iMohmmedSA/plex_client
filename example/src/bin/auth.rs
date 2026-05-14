use std::{fs, time::Duration};

use plex_client::client::{Client, crypto::DeviceKeypair};
use serde::Serialize;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let client_id = Uuid::new_v4().to_string();
    let client = Client::builder()
        .client_identifier(&client_id)
        .device("Plex Client example rust")
        .build()
        .expect("Failed building client");
    let keypair = DeviceKeypair::generate();

    let mut pin = client
        .request_pin(true, &keypair)
        .await
        .expect("Failed fetching pin");
    let auth_url = client.auth_url(&pin, None);

    println!("Login throgh: {}", auth_url);

    while pin.auth_token.is_none() {
        if pin.expires_in <= 0 {
            panic!("Auth link expired");
        }

        pin = client.poll_pin(pin.id, &keypair).await.unwrap();

        if pin.auth_token.is_some() {
            break;
        }

        println!("Waiting for your login...");
        sleep(Duration::from_secs(4)).await
    }

    let token = pin.auth_token.unwrap();

    println!("Your token: {}", token);
    save_login(token, client_id, keypair);
}

#[derive(Serialize)]
struct SavedLogin {
    token: String,
    client_id: String,

    // Keypair
    kid: String,
    private_key: [u8; 32],
}

fn save_login(token: String, client_id: String, keypair: DeviceKeypair) {
    let login = SavedLogin {
        token,
        client_id,
        kid: keypair.kid().to_string(),
        private_key: keypair.private_key_bytes(),
    };
    let json = serde_json::to_string_pretty(&login).expect("Failed to serialize");
    fs::write("login.json", json).expect("Failed to write login.json");

    println!("\n\n========================================================");
    println!("\nYou are ready to run the other examples.");
    println!("Your token is saved at login.json.");
    println!("Once you are done make sure to remove the access");
    println!("Visit https://app.plex.tv/desktop/#!/settings/devices/all");
}
