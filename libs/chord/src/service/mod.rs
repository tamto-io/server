use crate::client::ClientError;
use crate::node::store::{NodeStore, Db};
use crate::node::Finger;
use crate::{Client, Node, NodeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Mutex, Arc};

#[derive(Debug)]
pub struct NodeService<C: Client> {
    id: NodeId,
    addr: SocketAddr,
    store: NodeStore,
    phantom: PhantomData<C>,
    clients: Arc<Mutex<HashMap<NodeId, C>>>,
}

impl<C: Client + Clone> NodeService<C> {
    pub fn new(socket_addr: SocketAddr) -> Self {
        let id = socket_addr.into();
        Self::with_id(id, socket_addr)
    }

    fn with_id(id: NodeId, addr: SocketAddr) -> Self {
        let store = NodeStore::new(Node::with_id(id, addr));
        Self {
            id,
            addr,
            store,
            phantom: PhantomData,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub(crate) fn store(&self) -> Db {
        self.store.db()
    }

    pub(crate) fn client(&self, node: Node) -> C {
        let lock = self.clients.lock();
        if let Err(e) = &lock {
            log::error!("Failed to lock clients: {}", e);
        }
        let mut clients = lock.unwrap();
        if let Some(client) = clients.get(&node.id) {
            client.clone()
        } else {
            log::info!("Creating new client for node {}, endpoint: {}", node.id, node.addr);
            let client: C = node.client();
            clients.insert(node.id, client.clone());
            client
        }
    }

    /// Find the successor of the given id.
    ///
    /// If the given id is in the range of the current node and its successor, the successor is returned.
    /// Otherwise, the successor of the closest preceding node is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to find the successor for
    pub async fn find_successor(&self, id: NodeId) -> Result<Node, error::ServiceError> {
        let successor = self.store().successor();
        if Node::is_between_on_ring(id.0, self.id.0, successor.id.0) {
            Ok(successor.clone())
        } else {
            let n = self.closest_preceding_node(id);
            let client: C = self.client(n);
            let successor = client.find_successor(id).await?;
            Ok(successor)
        }
    }

    pub async fn get_predecessor(&self) -> Result<Option<Node>, error::ServiceError> {
        Ok(self.store().predecessor())
    }

    /// Join the chord ring.
    ///
    /// This method is used to join the chord ring. It will find the successor of its own id
    /// and set it as the successor.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to join the ring with. It's an existing node in the ring.
    pub async fn join(&self, node: Node) -> Result<(), error::ServiceError> {
        let client: C = self.client(node);
        let successor = client.find_successor(self.id).await?;
        self.store().set_successor(successor);

        Ok(())
    }

    /// Notify the node about a potential new predecessor.
    ///
    /// If the predecessor is not set or the given node is in the range of the current node and the
    /// predecessor, the predecessor is set to the given node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node which might be the new predecessor
    pub fn notify(&self, node: Node) {
        let predecessor = self.store().predecessor();
        if predecessor.is_none()
            || Node::is_between_on_ring(node.id.clone().0, predecessor.unwrap().id.0, self.id.0)
        {
            log::debug!("Setting predecessor to {:?}", node);
            {
                self.store().set_predecessor(node);
            }
        }
    }

    /// Stabilize the node
    ///
    /// This method is used to stabilize the node. It will check if a predecessor of the successor
    /// is in the range of the current node and its successor. If so, the successor will be set to
    /// the retrieved predecessor.
    ///
    /// It will also notify the successor about the current node.
    ///
    /// > **Note**
    /// >
    /// > This method should be called periodically.
    pub async fn stabilize(&self) -> Result<(), error::ServiceError> {
        let successor = self.store().successor();
        let client: C = self.client(successor);
        // let client: C = self.store().successor().client();
        let result = client.predecessor().await;
        if let Ok(Some(x)) = result {
            if Node::is_between_on_ring(x.id.clone().0, self.id.0, self.store().successor().id.0) {
                self.store().set_successor(x);
            }
        }

        let successor = self.store().successor();
        let client: C = self.client(successor);
        // let client: C = self.store().successor().client();
        client.notify(Node {
            id: self.id,
            addr: self.addr,
        }).await?;

        Ok(())
    }

    /// Check predecessor
    ///
    /// This method is used to check if the predecessor is still alive. If not, the predecessor is
    /// set to `None`.
    ///
    /// > **Note**
    /// >
    /// > This method should be called periodically.
    pub fn check_predecessor(&self) {
        if let Some(predecessor) = self.store().predecessor() {
            let client: C = self.client(predecessor);
            // let client: C = predecessor.client();
            if let Err(ClientError::ConnectionFailed(_)) = client.ping() {
                self.store().unset_predecessor();
            };
        }
    }

    /// Fix fingers
    ///
    /// This method is used to fix the fingers. It iterates over all fingers and re-requests the
    /// successor of the finger's id. Then sets the successor of the finger to the retrieved node.
    ///
    /// > **Note**
    /// >
    /// > This method should be called periodically.
    pub async fn fix_fingers(&self) {
        for i in 0..Finger::FINGER_TABLE_SIZE {
            let finger_id = Finger::finger_id(self.id.0, (i + 1) as u8);
            let result = { self.find_successor(NodeId(finger_id)).await };
            if let Ok(successor) = result {
                self.store().update_finger(i.into(), successor)
            } else {
                log::error!("Failed to fix finger: {:?}", result.unwrap_err());
            }
        }
    }

    /// Get finger table
    /// 
    /// This method is used to get the finger table of the node.
    pub fn finger_table(&self) -> Vec<Finger> {
        self.store().finger_table()
    }

    /// Get closest preceding node
    /// 
    /// This method is used to get the closest preceding node of the given id.
    /// It will iterate over the finger table and return the closest node to the given id.
    /// 
    /// # Arguments
    /// 
    /// * `id` - The id to find the closest preceding node for
    /// 
    /// # Returns
    /// 
    /// The closest preceding node
    fn closest_preceding_node(&self, id: NodeId) -> Node {
        self.store()
            .closest_preceding_node(self.id.0, id.0)
            .unwrap_or(Node::new(self.addr))
    }
}

pub mod error {
    use crate::client;
    use std::fmt::Display;

    #[derive(Debug)]
    pub enum ServiceError {
        Unexpected(String),
    }

    impl From<client::ClientError> for ServiceError {
        fn from(err: client::ClientError) -> Self {
            Self::Unexpected(format!("Client error: {}", err))
        }
    }

    impl Display for ServiceError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unexpected(message) => write!(f, "{}", message),
            }
        }
    }
}
