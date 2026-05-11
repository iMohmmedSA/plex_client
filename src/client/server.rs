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
    endpoints::{ResourcesRequest, ServerRequest},
    error::Result,
    models::resource::Resource,
};

// 2.5 min
const MAX_AGE: Duration = Duration::from_secs(150);

#[derive(Debug, Default, Clone, Copy)]
pub(crate) enum Status {
    #[default]
    Idle,
    Reachable,
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub(crate) url: String,
    pub(crate) token: Arc<String>,

    status: Status,
    is_https: bool,
    is_relay: bool,
    is_local: bool,
}

pub(crate) struct ServerTarget {
    pub(crate) server_id: String,
    pub(crate) connection_index: u8,
    pub(crate) connection: Connection,
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

    pub(crate) fn servers_targets(&self) -> Vec<ServerTarget> {
        let mut servers = self.servers.write();
        let mut targets = Vec::new();

        for server in servers.iter_mut() {
            server.best_connection_index = None;

            for (index, connection) in server.connections.iter_mut().enumerate() {
                connection.status = Status::Idle;
                let Ok(connection_index) = u8::try_from(index) else {
                    continue;
                };

                targets.push(ServerTarget {
                    server_id: server.id.clone(),
                    connection_index,
                    connection: connection.clone(),
                });
            }
        }

        targets
    }

    pub(crate) fn server_targets(&self, server_id: &str) -> Vec<ServerTarget> {
        let servers = self.servers.read();
        let Some(server) = servers.iter().find(|server| server.id == server_id) else {
            return vec![];
        };

        let mut targets: Vec<ServerTarget> = server
            .connections
            .iter()
            .enumerate()
            .filter_map(|(index, connection)| {
                let connection_index = u8::try_from(index).ok()?;

                if Some(connection_index) == server.best_connection_index {
                    return None;
                }

                Some(ServerTarget {
                    server_id: server_id.to_string(),
                    connection_index,
                    connection: connection.clone(),
                })
            })
            .collect();

        if let Some(best) = server.best_connection_index
            && let Some(connection) = server.connections.get(best as usize)
        {
            targets.insert(
                0,
                ServerTarget {
                    server_id: server_id.to_string(),
                    connection_index: best,
                    connection: connection.clone(),
                },
            );
        };

        targets
    }

    pub(crate) fn mark_server_result(&self, target: &ServerTarget, status: Status) {
        let mut server = self.servers.write();

        let Some(server) = server
            .iter_mut()
            .find(|server| server.id == target.server_id)
        else {
            return;
        };

        let Some(connection) = server.connections.get_mut(target.connection_index as usize) else {
            return;
        };

        if connection.url != target.connection.url {
            return;
        }

        connection.status = status;

        match status {
            Status::Reachable if server.best_connection_index.is_none() => {
                server.best_connection_index = Some(target.connection_index)
            }
            Status::Unreachable
                if server.best_connection_index == Some(target.connection_index) =>
            {
                server.best_connection_index = None
            }
            _ => (),
        }
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
        self.check_servers_reachable().await;

        Ok(())
    }

    async fn check_servers_reachable(&self) {
        let targets = self.inner.registry.servers_targets();

        for target in targets {
            let status = match self.get_identity(&target.connection).await.is_ok() {
                true => Status::Reachable,
                false => Status::Unreachable,
            };

            self.inner.registry.mark_server_result(&target, status);
        }
    }
}
