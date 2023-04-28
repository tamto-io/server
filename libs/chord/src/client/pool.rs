use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{Client, Node, NodeId};

#[derive(Debug)]
pub struct ClientsPool<C: Client> {
    clients: Arc<Mutex<HashMap<NodeId, Arc<C>>>>,
}

impl<C: Client> ClientsPool<C> {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the client for the given node.
    /// If the client is not yet initialized, it will be initialized.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to get the client for
    pub async fn get_or_init(&self, node: Node) -> Arc<C> {
        let client = {
            let state = self.clients.lock().unwrap();
            state.get(&node.id()).map(|c| c.clone())
        };

        match client {
            Some(c) => c,
            None => {
                log::debug!("Initializing client for node: {}", node.addr());
                let client = C::init(node.addr()).await;
                let client = Arc::new(client);
                {
                    let mut state = self.clients.lock().unwrap();
                    state.insert(node.id(), client.clone());
                }
                client
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockClient;
    use crate::Node;
    use std::net::SocketAddr;

    use lazy_static::lazy_static;
    use std::sync::{Mutex, MutexGuard};

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    // When a test panics, it will poison the Mutex. Since we don't actually
    // care about the state of the data we ignore that it is poisoned and grab
    // the lock regardless.  If you just do `let _m = &MTX.lock().unwrap()`, one
    // test panicking will cause all other tests that try and acquire a lock on
    // that Mutex to also panic.
    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[tokio::test]
    async fn test_getting_client() {
        let _m = get_lock(&MTX);
        let ctx = MockClient::init_context();

        ctx.expect()
            .returning(|_addr: SocketAddr| MockClient::new());

        let node = Node::new("[::1]:42012".parse().unwrap());

        let pool: ClientsPool<MockClient> = ClientsPool::new();
        {
            let clients = pool.clients.lock().unwrap();
            assert!(clients.is_empty());
        }

        pool.get_or_init(node.clone()).await;
        {
            let clients = pool.clients.lock().unwrap();
            assert_eq!(clients.len(), 1);
            assert!(clients.contains_key(&node.id()));
        }

        pool.get_or_init(node.clone()).await;
        {
            let clients = pool.clients.lock().unwrap();
            assert_eq!(clients.len(), 1);
            assert!(clients.contains_key(&node.id()));
        }
    }
}
