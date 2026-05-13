use std::sync::Arc;

use http::{
    HeaderMap, HeaderValue,
    header::{ACCEPT, USER_AGENT},
};

use crate::{
    client::{Client, inner::ClientInner},
    error::{Error, Result},
    headers::{
        CLIENT_ID, DEVICE, DEVICE_NAME, DEVICE_VENDOR, MARKETPLACE, MODEL, PLATFORM,
        PLATFORM_VERSION, PRODUCT, VERSION,
    },
};

#[derive(Default, Clone)]
pub struct ClientBuilder {
    client_id: Option<String>,
    token: Option<String>,
    product: Option<String>,
    version: Option<String>,
    platform: Option<String>,
    platform_version: Option<String>,
    device: Option<String>,
    device_vendor: Option<String>,
    device_name: Option<String>,
    model: Option<String>,
    marketplace: Option<String>,
}

impl ClientBuilder {
    pub fn client_identifier(mut self, id: impl Into<String>) -> Self {
        self.client_id = Some(id.into());
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn maybe_token(mut self, token: Option<impl Into<String>>) -> Self {
        self.token = token.map(|t| t.into());
        self
    }

    pub fn product(mut self, product: impl Into<String>) -> Self {
        self.product = Some(product.into());
        self
    }

    pub fn maybe_product(mut self, product: Option<impl Into<String>>) -> Self {
        self.product = product.map(|p| p.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn maybe_version(mut self, version: Option<impl Into<String>>) -> Self {
        self.version = version.map(|v| v.into());
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn maybe_platform(mut self, platform: Option<impl Into<String>>) -> Self {
        self.platform = platform.map(|p| p.into());
        self
    }

    pub fn platform_version(mut self, platform_version: impl Into<String>) -> Self {
        self.platform_version = Some(platform_version.into());
        self
    }

    pub fn maybe_platform_version(mut self, platform_version: Option<impl Into<String>>) -> Self {
        self.platform_version = platform_version.map(|v| v.into());
        self
    }

    pub fn device(mut self, device: impl Into<String>) -> Self {
        self.device = Some(device.into());
        self
    }

    pub fn maybe_device(mut self, device: Option<impl Into<String>>) -> Self {
        self.device = device.map(|d| d.into());
        self
    }

    pub fn device_vendor(mut self, device_vendor: impl Into<String>) -> Self {
        self.device_vendor = Some(device_vendor.into());
        self
    }

    pub fn maybe_device_vendor(mut self, device_vendor: Option<impl Into<String>>) -> Self {
        self.device_vendor = device_vendor.map(|v| v.into());
        self
    }

    pub fn device_name(mut self, device_name: impl Into<String>) -> Self {
        self.device_name = Some(device_name.into());
        self
    }

    pub fn maybe_device_name(mut self, device_name: Option<impl Into<String>>) -> Self {
        self.device_name = device_name.map(|n| n.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn maybe_model(mut self, model: Option<impl Into<String>>) -> Self {
        self.model = model.map(|m| m.into());
        self
    }

    pub fn marketplace(mut self, marketplace: impl Into<String>) -> Self {
        self.marketplace = Some(marketplace.into());
        self
    }

    pub fn maybe_marketplace(mut self, marketplace: Option<impl Into<String>>) -> Self {
        self.marketplace = marketplace.map(|m| m.into());
        self
    }

    pub fn build(self) -> Result<Client> {
        let client_id = self
            .client_id
            .ok_or_else(|| Error::Generic("client_identifier is required".to_string()))?;

        let product = self.product;
        let version = self.version;
        let platform = self.platform;
        let platform_version = self.platform_version;
        let device = self.device;
        let device_vendor = self.device_vendor;
        let device_name = self.device_name;
        let model = self.model;
        let marketplace = self.marketplace;

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        // Required
        headers.insert(CLIENT_ID, HeaderValue::from_str(&client_id)?);

        // Helper to insert only when Some
        let mut insert_opt = |name: &'static str, value: Option<&str>| -> Result<()> {
            if let Some(v) = value {
                headers.insert(name, HeaderValue::from_str(v)?);
            }
            Ok(())
        };

        // Optional Plex headers
        insert_opt(PRODUCT, product.as_deref())?;
        insert_opt(VERSION, version.as_deref())?;
        insert_opt(PLATFORM, platform.as_deref())?;
        insert_opt(PLATFORM_VERSION, platform_version.as_deref())?;
        insert_opt(DEVICE, device.as_deref())?;
        insert_opt(DEVICE_VENDOR, device_vendor.as_deref())?;
        insert_opt(DEVICE_NAME, device_name.as_deref())?;
        insert_opt(MODEL, model.as_deref())?;
        insert_opt(MARKETPLACE, marketplace.as_deref())?;

        // User-Agent: "{product}/{version} - {crate_name}/{crate_version}"
        let lib_name: &str = env!("CARGO_PKG_NAME");
        let lib_version: &str = env!("CARGO_PKG_VERSION");
        let lib_ident = format!("{}/{}", lib_name, lib_version);
        let user_agent = match (product.as_deref(), version.as_deref()) {
            (Some(p), Some(v)) => format!("{}/{} - {}", p, v, lib_ident),
            (Some(p), None) => format!("{} - {}", p, lib_ident),
            (None, Some(v)) => format!("{} - {}", v, lib_ident),
            (None, None) => lib_ident,
        };
        headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent)?);

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let inner = Arc::new(ClientInner {
            reqwest: http_client,
            token: self.token,
            registry: Default::default(),
            client_id,
            product,
        });

        Ok(Client { inner })
    }
}
