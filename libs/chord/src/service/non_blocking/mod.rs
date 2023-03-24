use std::{marker::PhantomData, net::SocketAddr};

use seahash::hash;

use crate::{node::store::NodeStore, Client, Node, NodeService};

impl<C: Client> NodeService<C> {
    pub fn new(socket_addr: SocketAddr) -> Self {
        let id = hash(socket_addr.ip().to_string().as_bytes());
        Self::with_id(id, socket_addr)
    }

    fn with_id(id: u64, addr: SocketAddr) -> Self {
        let store = NodeStore::new(Node::with_id(id, addr));
        Self {
            id,
            addr,
            store,
            phantom: PhantomData,
        }
    }
}
