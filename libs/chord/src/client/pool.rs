use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{Client, Node, NodeId};

#[derive(Debug)]
pub struct ClientsPool<C: Client> {
    clients: Arc<Mutex<HashMap<NodeId, Arc<C>>>>,
}

impl<C: Client> Default for ClientsPool<C> {
    fn default() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<C: Client> ClientsPool<C> {
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
    use crate::service::tests::MTX;
    use crate::Node;
    use crate::{client::MockClient, service::tests::get_lock};

    #[tokio::test]
    async fn test_getting_client() {
        let _m = get_lock(&MTX);
        let ctx = MockClient::init_context();

        ctx.expect().returning(|_| MockClient::new());

        let node = Node::new("[::1]:42080".parse().unwrap());

        let pool: ClientsPool<MockClient> = ClientsPool::default();
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
