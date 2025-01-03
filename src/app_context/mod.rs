use crate::storage::interface::IRoomStorage;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::storage::sockets::HashMapClientSocketsStorage;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;

#[derive(Clone, Default)]
pub struct AppContext<RS: IRoomStorage> {
    pub rooms: RS,
    // TODO: make the struct generic over sockets storage as well?
    pub sockets: HashMapClientSocketsStorage,
}

#[derive(Clone)]

pub struct RequestContext {
    pub public_id: String,
    pub private_id: String,
    pub room_id: String,
    // TODO: add the `client_ip` back so that IP is logges on websocket messages as well
    // pub client_ip: Option<SocketAddr>,
}

pub fn init() -> AppContext<HashMapRoomsStorage> {
    let app_context = AppContext::<HashMapRoomsStorage>::default();
    let app_context_in_sockets_logger = app_context.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let count = app_context_in_sockets_logger.sockets.count().await;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            tracing::info!(task = "sockets_count", count, timestamp);
        }
    });
    app_context
}
