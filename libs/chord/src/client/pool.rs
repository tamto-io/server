use std::{collections::HashMap, sync::{Arc, Mutex}};

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

    pub async fn get_or_init(&self, node: Node) -> Result<Arc<C>, ClientsPoolError> {
        let client = {
            let state = self.clients.lock().unwrap();
            state.get(&node.id()).map(|c| c.clone())
        };

        match client {
            Some(c) => Ok(c),
            None => {
                log::debug!("Initializing client for node: {}", node.addr());
                let client = C::init(node.addr()).await;
                let client = Arc::new(client);
                {
                    let mut state = self.clients.lock().unwrap();
                    state.insert(node.id(), client.clone());
                }
                Ok(client)
            }
        }
    }
}

#[derive(Debug)]
pub enum ClientsPoolError {
    ClientNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NodeId, Node};
    use crate::client::{Client, MockClient};
    use crate::service::error::ServiceError;
    use std::net::SocketAddr;

    use crate::node::store::NodeStore;
    use crate::node::Finger;
    use lazy_static::lazy_static;
    use mockall::predicate;
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

        ctx.expect().returning(|_addr: SocketAddr| {
            MockClient::new()
        });

        let node = Node::new("[::1]:42012".parse().unwrap());

        let pool: ClientsPool<MockClient> = ClientsPool::new();
        {
            let clients = pool.clients.lock().unwrap();
            assert!(clients.is_empty());
        }

        pool.get_or_init(node.clone()).await.unwrap();
        {
            let clients = pool.clients.lock().unwrap();
            assert_eq!(clients.len(), 1);
            assert!(clients.contains_key(&node.id()));
        }

        pool.get_or_init(node.clone()).await.unwrap();
        {
            let clients = pool.clients.lock().unwrap();
            assert_eq!(clients.len(), 1);
            assert!(clients.contains_key(&node.id()));
        }
    }
}
