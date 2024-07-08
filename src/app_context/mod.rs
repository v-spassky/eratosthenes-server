use crate::storage::rooms::Rooms;
use crate::storage::sockets::ClientSockets;

#[derive(Clone, Default)]
pub struct AppContext {
    pub rooms: Rooms,
    pub sockets: ClientSockets,
}
