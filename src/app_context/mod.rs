use crate::storage::interface::IRoomStorage;
use crate::storage::sockets::HashMapClientSocketsStorage;

#[derive(Clone, Default)]
pub struct AppContext<RS: IRoomStorage> {
    pub rooms: RS,
    pub sockets: HashMapClientSocketsStorage,
}
