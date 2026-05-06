use crate::client::server::ServerRegistry;

#[derive(Debug)]
pub(crate) struct ClientInner {
    pub(crate) reqwest: reqwest::Client,
    pub(crate) token: Option<String>,

    pub(crate) registry: ServerRegistry,

    pub(crate) client_id: String,
    pub(crate) product: Option<String>,
}
