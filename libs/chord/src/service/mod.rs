use crate::client::ClientError;
use crate::node::store::{NodeStore, Db};
use crate::node::Finger;
use crate::{Client, Node};
use seahash::hash;
use std::marker::PhantomData;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct NodeService<C: Client> {
    id: u64,
    addr: SocketAddr,
    store: NodeStore,
    phantom: PhantomData<C>,
}

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

    pub fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn store(&self) -> Db {
        self.store.db()
    }

    /// Find the successor of the given id.
    ///
    /// If the given id is in the range of the current node and its successor, the successor is returned.
    /// Otherwise, the successor of the closest preceding node is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to find the successor for
    pub async fn find_successor(&self, id: u64) -> Result<Node, error::ServiceError> {
        let successor = self.store().successor();
        if Node::is_between_on_ring(id, self.id, successor.id) {
            Ok(successor.clone())
        } else {
            let n = self.closest_preceding_node(id);
            let client: C = n.client();
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
        let client: C = node.client();
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
            || Node::is_between_on_ring(node.id.clone(), predecessor.unwrap().id, self.id)
        {
            self.store().set_predecessor(node);
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
        let client: C = self.store().successor().client();
        let result = client.predecessor().await;
        if let Ok(Some(x)) = result {
            if Node::is_between_on_ring(x.id.clone(), self.id, self.store().successor().id) {
                self.store().set_successor(x);
            }
        }

        let client: C = self.store().successor().client();
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
            let client: C = predecessor.client();
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
            let finger_id = Finger::finger_id(self.id, (i + 1) as u8);
            let result = { self.find_successor(finger_id).await };
            if let Ok(successor) = result {
                self.store().update_finger(i.into(), successor)
                // self.store().finger_table[i].node = successor;
            }
        }
    }

    /// Get finger table
    /// 
    /// This method is used to get the finger table of the node.
    pub fn finger_table(&self) -> Vec<Finger> {
        self.store().finger_table()
    }

    fn closest_preceding_node(&self, id: u64) -> Node {
        self.store().closest_preceding_node(self.id, id)
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
