mod pool;

use crate::{Node, NodeId};
use async_trait::async_trait;
use error_stack::Result;
use mockall::automock;
pub use pool::ClientsPool;
use thiserror::Error;
use std::net::SocketAddr;

#[automock]
#[async_trait]
pub trait Client {
    /// Init the client
    ///
    /// # Arguments
    ///
    /// * `addr` - The node address to connect to
    async fn init(addr: SocketAddr) -> Self;

    /// Find a successor of a given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to find the successor for
    async fn find_successor(&self, id: NodeId) -> Result<Node, ClientError>;

    /// Get the successor of the node
    async fn successor(&self) -> Result<Node, ClientError>;

    /// Get successor list of the node
    async fn successor_list(&self) -> Result<Vec<Node>, ClientError>;

    /// Get the predecessor of the node
    async fn predecessor(&self) -> Result<Option<Node>, ClientError>;

    /// Notify the node about a new predecessor
    ///
    /// # Arguments
    ///
    /// * `predecessor` - The new predecessor
    async fn notify(&self, predecessor: Node) -> Result<(), ClientError>;

    /// Ping the node
    async fn ping(&self) -> Result<(), ClientError>;
}

#[derive(Debug, Clone, Error)]
pub enum ClientError {
    #[error("{0}")]
    ConnectionFailed(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Client not initialized")]
    NotInitialized,
    #[error("Unexpected error")]
    Unexpected,

    #[error("Ping failed")]
    PingFailed,
    #[error("Find successor failed")]
    FindSuccessorFailed,
    #[error("Get successor failed")]
    GetSuccessorFailed,
    #[error("Get successor list failed")]
    GetSuccessorListFailed,
    #[error("Get predecessor failed")]
    GetPredecessorFailed,
    #[error("Notify failed")]
    NotifyFailed,
}

#[cfg(test)]
impl Clone for MockClient {
    fn clone(&self) -> Self {
        Self::default()
    }
}
