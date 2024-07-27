use crate::storage::interface::IRoomStorage;
use crate::storage::sockets::HashMapClientSocketsStorage;
use std::net::SocketAddr;

#[derive(Clone, Default)]
pub struct AppContext<RS: IRoomStorage> {
    pub rooms: RS,
    pub sockets: HashMapClientSocketsStorage,
}

pub struct RequestContext {
    pub public_id: String,
    pub private_id: String,
    pub room_id: String,
    pub client_ip: Option<SocketAddr>,
}
