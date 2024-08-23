use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::Message;

pub static NEXT_SOCKET_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Default)]
pub struct HashMapClientSocketsStorage {
    storage: Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>>,
}

impl HashMapClientSocketsStorage {
    pub async fn add(&self, socket: mpsc::UnboundedSender<Message>) -> usize {
        let socket_id = NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed);
        self.storage.write().await.insert(socket_id, socket);
        socket_id
    }

    pub async fn remove(&self, socket_id: usize) {
        self.storage.write().await.remove(&socket_id);
    }

    pub async fn send_msg(&self, msg: &str, socket_id: usize) {
        if let Err(_disconnected) = self
            .storage
            .read()
            .await
            .get(&socket_id)
            .unwrap()
            .send(Message::text(msg))
        {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
            eprintln!("[user_message]: error sending pong to user: {socket_id:?}")
        }
    }

    // TODO: refactor so that `msg` is a struct or enum, not `&str`
    pub async fn broadcast_msg(&self, msg: &str, sockets_ids: &[Option<usize>]) {
        for (&uid, tx) in self.storage.read().await.iter() {
            if sockets_ids.contains(&Some(uid)) {
                if let Err(_disconnected) = tx.send(Message::text(msg)) {
                    // The tx is disconnected, our `user_disconnected` code
                    // should be happening in another task, nothing more to
                    // do here.
                    eprintln!(
                        "[user_message]: error broadcasting message {msg} to user ith id {uid:?}"
                    );
                }
            }
        }
    }

    pub async fn count(&self) -> usize {
        self.storage.read().await.len()
    }
}
