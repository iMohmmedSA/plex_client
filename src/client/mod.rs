pub mod auth;
pub mod builder;
pub mod crypto;
mod inner;
pub mod server;

use std::sync::Arc;

use crate::{
    client::{
        builder::ClientBuilder,
        inner::ClientInner,
        server::{Connection, Status},
    },
    error::{Error, Result},
    headers::TOKEN,
};
use serde::{Serialize, de::DeserializeOwned};

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

#[derive(Clone, Copy)]
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
        Q: Serialize,
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
        B: Serialize,
        Q: Serialize,
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
        B: Serialize,
        Q: Serialize,
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

    pub(crate) async fn request_server<T, B, Q>(
        &self,
        server_id: &str,
        method: reqwest::Method,
        prefix: Api,
        path: &str,
        body: Option<B>,
        query: Option<Q>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + Clone,
        Q: Serialize + Clone,
    {
        self.ensure_servers_fresh().await?;
        let targets = self.inner.registry.server_targets(server_id);
        let mut last_transport_error = None;

        for target in targets {
            let result = self
                .request(
                    method.clone(),
                    Base::Connection(target.connection.clone()),
                    prefix,
                    path,
                    body.clone(),
                    query.clone(),
                )
                .await;

            match result {
                Ok(value) => {
                    self.inner
                        .registry
                        .mark_server_result(&target, Status::Reachable);
                    return Ok(value);
                }
                Err(error) if is_transport_failure(&error) => {
                    last_transport_error = Some(error);
                    self.inner
                        .registry
                        .mark_server_result(&target, Status::Unreachable);
                }
                Err(error) => return Err(error),
            }
        }

        if let Some(error) = last_transport_error {
            return Err(error);
        }

        return Err(Error::Generic(format!(
            "Server ({}) has no connections",
            server_id
        )));
    }
}

fn is_transport_failure(error: &Error) -> bool {
    match error {
        Error::Network(e) => {
            e.is_connect()
                || e.is_timeout()
                || (e.status().is_none() && !e.is_decode() && !e.is_builder() && !e.is_redirect())
        }
        _ => false,
    }
}
