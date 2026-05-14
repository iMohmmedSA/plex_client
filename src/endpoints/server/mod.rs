use crate::{
    client::{Api, Base, Client, server::Connection},
    error::Result,
    models::server::{Identity, MediaContainer},
};

#[allow(async_fn_in_trait)]
pub trait ServerRequest {
    async fn get_identity_with(&self, id: &str) -> Result<MediaContainer<Identity>>;
    async fn get_identity(&self, connection: &Connection) -> Result<MediaContainer<Identity>>;
}

impl ServerRequest for Client {
    async fn get_identity_with(&self, server_id: &str) -> Result<MediaContainer<Identity>> {
        self.get_server(server_id, Api::Raw, "/identity", None::<()>)
            .await
    }

    async fn get_identity(&self, connection: &Connection) -> Result<MediaContainer<Identity>> {
        self.get(
            Base::Connection(connection.clone()),
            Api::Raw,
            "/identity",
            None::<()>,
        )
        .await
    }
}
