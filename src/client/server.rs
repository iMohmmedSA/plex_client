use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use parking_lot::RwLock;

use crate::{
    client::Client,
    endpoints::ResourcesRequest,
    error::Result,
    models::resource::{Connection as ResourceConnection, Resource},
};

// 2.5 min
const MAX_AGE: Duration = Duration::from_secs(150);

#[derive(Debug, Default)]
pub(crate) enum Status {
    #[default]
    Idle,
    Reachable,
    Unreachable,
}

#[derive(Debug)]
pub struct Connection {
    pub(crate) url: String,
    pub(crate) token: Arc<String>,

    status: Status,
    is_https: bool,
    is_relay: bool,
    is_local: bool,
}

pub(crate) struct ProbeTarget {
    pub(crate) server_id: String,
    pub(crate) connection_index: u8,
    pub(crate) url: String,
    pub(crate) token: Arc<String>,
}

#[derive(Debug)]
pub(crate) struct Server {
    id: String,
    token: Arc<String>,

    best_connection_index: Option<u8>,
    connections: Vec<Connection>,
}

#[derive(Debug, Default)]
pub(crate) struct ServerRegistry {
    servers: RwLock<Vec<Server>>,
    last_refreshed: RwLock<Option<Instant>>,
    refreshing: AtomicBool,
}

impl Connection {
    fn rank(&self) -> u8 {
        match (self.is_local, self.is_relay, self.is_https) {
            (true, _, true) => 0,
            (true, _, false) => 1,
            (false, false, true) => 2,
            (false, false, false) => 3,
            (false, true, true) => 4,
            (false, true, false) => 5,
        }
    }
}

impl Server {
    fn from_resources(rs: Vec<Resource>) -> Vec<Self> {
        rs.into_iter()
            .filter_map(|r| {
                let token = Arc::new(r.access_token?);
                let mut connections = r
                    .connections
                    .into_iter()
                    .map(|c| Connection {
                        url: c.uri,
                        token: token.clone(),
                        status: Default::default(),
                        is_https: c.protocol.eq_ignore_ascii_case("https"),
                        is_relay: c.relay,
                        is_local: c.local,
                    })
                    .collect::<Vec<_>>();
                connections.sort_by_key(Connection::rank);

                Some(Self {
                    id: r.client_identifier,
                    token,
                    best_connection_index: None,
                    connections,
                })
            })
            .collect()
    }
}

impl ServerRegistry {
    pub(crate) fn refresh(&self, resources: Vec<Resource>) {
        let mut servers = self.servers.write();
        *servers = Server::from_resources(resources);

        let mut last = self.last_refreshed.write();
        *last = Some(Instant::now());

        self.release_refreshing();
    }

    pub(crate) fn needs_refresh(&self) -> bool {
        if self.refreshing.load(Ordering::Acquire) {
            return false;
        }

        let stale = match *self.last_refreshed.read() {
            None => true,
            Some(t) => t.elapsed() > MAX_AGE,
        };

        if !stale {
            return false;
        }

        self.refreshing
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub(crate) fn release_refreshing(&self) {
        self.refreshing.store(false, Ordering::Release);
    }

    pub(crate) fn probe_targets(&self) -> Vec<ProbeTarget> {
        let mut servers = self.servers.write();
        let mut targets = Vec::new();

        for server in servers.iter_mut() {
            server.best_connection_index = None;

            for (index, connection) in server.connections.iter_mut().enumerate() {
                connection.status = Status::Idle;
                let Ok(connection_index) = u8::try_from(index) else {
                    continue;
                };

                targets.push(ProbeTarget {
                    server_id: server.id.clone(),
                    connection_index,
                    url: connection.url.clone(),
                    token: server.token.clone(),
                });
            }
        }

        targets
    }
}

impl Client {
    pub(crate) async fn ensure_servers_fresh(&self) -> Result<()> {
        if !self.inner.registry.needs_refresh() {
            return Ok(());
        }

        // TODO: We should have it configurable.
        // TODO: We need to add feature tokio.
        // we will run this in background if we already have
        // servers that are reachable
        let resources = match self.get_resources(true, true, true).await {
            Ok(r) => r,
            Err(e) => {
                self.inner.registry.release_refreshing();
                return Err(e);
            }
        };

        self.inner.registry.refresh(resources);
        self.check_servers_reachable().await?; // TODO: what happens if it fails? does this mean ensure_servers_fresh fails too?

        Ok(())
    }

    async fn check_servers_reachable(&self) -> Result<()> {
        let targets = self.inner.registry.probe_targets();

        Ok(())
    }
}
