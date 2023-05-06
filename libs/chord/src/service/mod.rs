use async_recursion::async_recursion;
use error_stack::{Result, ResultExt, Report};

use crate::client::{ClientError, ClientsPool};
use crate::node::store::{Db, NodeStore};
use crate::node::Finger;
use crate::{Client, Node, NodeId};
use std::net::SocketAddr;
use std::sync::Arc;
use std::vec;

#[cfg(test)]
pub(crate) mod tests;

#[derive(Debug)]
pub struct NodeService<C: Client> {
    id: NodeId,
    addr: SocketAddr,
    store: NodeStore,

    clients: ClientsPool<C>,
}

impl<C: Client + Clone + Sync + Send + 'static> NodeService<C> {
    /// Create a new node service
    ///
    /// # Arguments
    ///
    /// * `socket_addr` - The address of the node
    /// * `replication_factor` - The number of successors to keep track of
    pub fn new(socket_addr: SocketAddr, replication_factor: usize) -> Self {
        let id: NodeId = socket_addr.into();
        Self::with_id(id, socket_addr, replication_factor)
    }

    fn with_id(id: impl Into<NodeId>, addr: SocketAddr, replication_factor: usize) -> Self {
        let id = id.into();
        let store = NodeStore::new(Node::with_id(id, addr), replication_factor);
        Self {
            id,
            addr,
            store,
            clients: ClientsPool::default(),
        }
    }

    pub fn id(&self) -> NodeId {
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
    pub async fn find_successor(&self, id: NodeId) -> Result<Node, error::ServiceError> {
        if let Some(successor) = self.find_immediate_successor(id).await? {
            Ok(successor)
        } else {
            self.find_successor_using_finger_table(id, None).await
        }
    }

    /// Find the successor of the given id using the successor list.
    async fn find_immediate_successor(
        &self,
        id: NodeId,
    ) -> Result<Option<Node>, error::ServiceError> {
        let successors = self.store().successor_list();
        for successor in successors {
            if Node::is_between_on_ring(id.0, self.id.0, successor.id.0) {
                return Ok(Some(successor));
            }
        }

        Ok(None)
    }

    /// Find the successor of the given id using the finger table.
    /// This method is called recursively until the successor is found or until the closest preceding node is the current node.
    ///
    /// If a node fails to respond, it's id is used to find new closest preceding node.
    /// If all nodes fail to respond, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to find the successor for
    /// * `failing_node` - The id of the node that failed to respond. It is used to find the new closest preceding node.    
    #[async_recursion]
    async fn find_successor_using_finger_table(
        &self,
        id: NodeId,
        failing_node: Option<NodeId>,
    ) -> Result<Node, error::ServiceError> {
        let search_id = failing_node.unwrap_or(id);
        let n = self.closest_preceding_node(search_id);

        if n.id == self.id {
            let error = format!("Cannot find successor of id '{}' using finger table", id);
            log::error!("{}", error);
            return Err(Report::new(error::ServiceError::Unexpected(error)));
        }

        let client: Arc<C> = self.client(&n).await;
        match client.find_successor(id).await {
            Ok(successor) => Result::Ok(successor),
            // Err(ClientError::ConnectionFailed(_)) => {
            //     self.find_successor_using_finger_table(id, Some(n.id)).await
            // }
            Err(report) => {
                match (*report.current_context()).clone() {
                    ClientError::ConnectionFailed(_) => {
                        self.find_successor_using_finger_table(id, Some(n.id)).await.change_context(error::ServiceError::FixMe)
                    }
                    err => {
                        Result::Err(report.change_context(err.into()))
                        // Err(err.into())
                        // let error = format!(
                        //     "Failed to find successor of id '{}' using finger table",
                        //     id
                        // );
                        // log::error!("{}", error);
                        // Err(error::ServiceError::Unexpected(error))
                    }
                }
            },
        }
    }

    pub async fn get_predecessor(&self) -> Result<Option<Node>, error::ServiceError> {
        Ok(self.store().predecessor())
    }

    pub async fn get_successor(&self) -> Result<Node, error::ServiceError> {
        Ok(self.store().successor())
    }

    pub async fn get_successor_list(&self) -> Result<Vec<Node>, error::ServiceError> {
        Ok(self.store().successor_list())
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
        let client: Arc<C> = self.client(&node).await;
        let successor = client.find_successor(self.id).await.change_context(error::ServiceError::FixMe)?;
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
            || Node::is_between_on_ring(node.id.0, predecessor.unwrap().id.0, self.id.0)
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
        let successor = self.store().successor();
        let client: Arc<C> = self.client(&successor).await;
        let result = client.predecessor().await;
        drop(client);

        if let Ok(Some(x)) = result {
            if Node::is_between_on_ring(x.id.0, self.id.0, self.store().successor().id.0) {
                self.store().set_successor(x);
            }
        }

        let successor = self.store().successor();
        let client: Arc<C> = self.client(&successor).await;

        client
            .notify(Node {
                id: self.id,
                addr: self.addr,
            })
            .await.change_context(error::ServiceError::FixMe)?;

        Ok(())
    }

    pub async fn reconcile_successors(&self) {
        let successor = self.store().successor();
        let client: Arc<C> = self.client(&successor).await;

        match client.successor_list().await {
            Ok(successors) => {
                let mut new_successors = vec![successor];
                new_successors.extend(successors);

                self.store().set_successor_list(new_successors);
            }
            Err(err) => {
                log::info!("Successor {:?} is down, removing from the successor list", successor.addr);
                log::debug!("Successor {:?} error: {err:?}", successor.addr);

                let successors = self.store().successor_list();
                self.store().set_successor_list(successors[1..].to_vec());
            },
        }
    }

    /// Check predecessor
    ///
    /// This method is used to check if the predecessor is still alive. If not, the predecessor is
    /// set to `None`.
    ///
    /// > **Note**
    /// >
    /// > This method should be called periodically.
    pub async fn check_predecessor(&self) -> Result<(), error::ServiceError> {
        if let Some(predecessor) = self.store().predecessor() {
            let client: Arc<C> = self.client(&predecessor).await;
            match client.ping().await {
                Ok(_) => Ok(()),
                Err(err) => {
                    log::info!("Predecessor {:?} is down, removing. Error: {:?}", predecessor.addr, err);
                    self.store().unset_predecessor();
                    Ok(())
                },
            }
        } else {
            Ok(())
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
            .unwrap_or(Node::with_id(self.id, self.addr))
    }

    async fn client(&self, node: &Node) -> Arc<C> {
        self.clients.get_or_init(node).await
    }
}

pub mod error {
    use error_stack::Context;

    use crate::client;
    use std::fmt::Display;

    #[derive(Debug)]
    pub enum ServiceError {
        Unexpected(String),
        ClientDisconnected,

        FixMe,
    }

    impl Context for ServiceError {}

    impl From<client::ClientError> for ServiceError {
        fn from(err: client::ClientError) -> Self {
            match err {
                client::ClientError::ConnectionFailed(_) => Self::ClientDisconnected,
                _ => Self::Unexpected(format!("Client error: {}", err)),
            }
        }
    }

    impl Display for ServiceError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unexpected(message) => write!(f, "{}", message),
                Self::ClientDisconnected => write!(f, "Client disconnected"),
                Self::FixMe => write!(f, "Fix me"),
            }
        }
    }
}
