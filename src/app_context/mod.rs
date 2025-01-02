use crate::storage::interface::IRoomStorage;
use crate::storage::rooms::HashMapRoomsStorage;
use crate::storage::sockets::HashMapClientSocketsStorage;
// use std::net::SocketAddr;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;

#[derive(Clone, Default)]
pub struct AppContext<RS: IRoomStorage> {
    pub rooms: RS,
    pub sockets: HashMapClientSocketsStorage,
}

#[derive(Clone)]

pub struct RequestContext {
    pub public_id: String,
    pub private_id: String,
    pub room_id: String,
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
