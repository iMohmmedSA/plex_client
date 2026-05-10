pub mod auth;
pub mod builder;
pub mod crypto;
mod inner;
pub mod server;

use std::sync::Arc;

use crate::{
    client::{builder::ClientBuilder, inner::ClientInner, server::Connection},
    error::{Error, Result},
    headers::TOKEN,
};
use serde::de::DeserializeOwned;

pub(crate) enum Base {
    Plex,                   // plex.tv
    Clients,                // clients.plex.tv
    Connection(Connection), // https://{ip}.{serverid}.plex.direct:32400
}

impl Base {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Base::Plex => "https://plex.tv",
            Base::Clients => "https://clients.plex.tv",
            Base::Connection(c) => &c.url,
        }
    }
}

pub(crate) enum Api {
    Raw,
    V2,
}

impl Api {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Api::Raw => "",
            Api::V2 => "/api/v2",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) inner: Arc<ClientInner>,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    pub fn is_authenticated(&self) -> bool {
        self.inner.token.is_some()
    }

    pub(crate) async fn get<T, Q>(
        &self,
        base: Base,
        api: Api,
        path: &str,
        query: Option<Q>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: serde::Serialize,
    {
        self.request(
            reqwest::Method::GET,
            base,
            api,
            path,
            None::<serde_json::Value>,
            query,
        )
        .await
    }

    pub(crate) async fn post<T, B, Q>(
        &self,
        base: Base,
        api: Api,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize,
        Q: serde::Serialize,
    {
        self.request(reqwest::Method::POST, base, api, path, body, query)
            .await
    }

    pub(crate) async fn request<T, B, Q>(
        &self,
        method: reqwest::Method,
        base: Base,
        prefix: Api,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: serde::Serialize,
        Q: serde::Serialize,
    {
        let url = format!("{}{}{}", base.as_str(), prefix.as_str(), path);
        let mut request = self.inner.reqwest.request(method, &url);

        if let Some(query) = query {
            request = request.query(&query);
        }

        if let Base::Connection(connection) = base {
            request = request.header(TOKEN, &(*connection.token));
        } else if let Some(token) = &self.inner.token {
            request = request.header(TOKEN, token);
        }

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api { status, message });
        }

        Ok(response.json().await?)
    }
}
