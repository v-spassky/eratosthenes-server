use crate::models::Room;
use std::collections::HashMap;
use std::sync::{atomic::AtomicUsize, Arc};
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

pub type ClientSockets = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>;
pub type Rooms = Arc<RwLock<HashMap<String, Room>>>;
