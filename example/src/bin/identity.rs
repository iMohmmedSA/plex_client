use std::fs;

use plex_client::{
    client::Client,
    endpoints::{ResourcesRequest, ServerRequest},
};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let login = SavedLogin::load();
    let client = Client::builder()
        .client_identifier(login.client_id)
        .token(login.token)
        .build()
        .unwrap();

    let resources = client
        .get_resources(true, true, true)
        .await
        .unwrap_or_else(|e| panic!("{e}"));

    for res in resources {
        println!(
            "{:?}",
            client.get_identity_with(&res.client_identifier).await
        );
    }
}

#[derive(Deserialize)]
struct SavedLogin {
    token: String,
    client_id: String,
}

impl SavedLogin {
    fn load() -> Self {
        let s = fs::read_to_string("login.json")
            .expect("You are missing login.json. run auth.rs first");
        serde_json::from_str(&s).expect("Delete login.json then run auth again.")
    }
}
