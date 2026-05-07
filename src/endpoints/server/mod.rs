use crate::{
    client::{Api, Base, Client},
    error::Result,
    models::{
        resource::Connection,
        server::{Identity, MediaContainer},
    },
};

#[allow(async_fn_in_trait)]
pub trait ServerRequest {
    async fn get_identity(&self, connection: Connection) -> Result<MediaContainer<Identity>>;
}

impl ServerRequest for Client {
    async fn get_identity(&self, connection: Connection) -> Result<MediaContainer<Identity>> {
        self.get(
            Base::Connection(connection),
            Api::Raw,
            "/identity",
            None::<()>,
        )
        .await
    }
}
