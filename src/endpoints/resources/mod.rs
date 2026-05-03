use std::collections::HashMap;

use crate::{
    client::{Api, Base, Client},
    error::{Error, Result},
    models::resource::Resource,
};

#[allow(async_fn_in_trait)]
pub trait ResourcesRequest {
    async fn get_resources(
        &self,
        include_https: bool,
        include_relay: bool,
        include_ipv6: bool,
    ) -> Result<Vec<Resource>>;
}

impl ResourcesRequest for Client {
    async fn get_resources(
        &self,
        include_https: bool,
        include_relay: bool,
        include_ipv6: bool,
    ) -> Result<Vec<Resource>> {
        if !self.is_authenticated() {
            return Err(Error::Generic("Missing authentication key".to_string()));
        }

        let mut params = HashMap::new();
        params.insert("includeHttps", include_https as u8);
        params.insert("includeRelay", include_relay as u8);
        params.insert("includeIPv6", include_ipv6 as u8);

        self.get(Base::Clients, Api::V2, "/resources", Some(params))
            .await
    }
}
