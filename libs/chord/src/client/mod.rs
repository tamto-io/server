mod pool;

use crate::{Node, NodeId};
use async_trait::async_trait;
use mockall::automock;
pub use pool::ClientsPool;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use tokio::sync::oneshot::error::RecvError;

#[automock]
#[async_trait]
pub trait Client {
    /// Init the client
    ///
    /// # Arguments
    ///
    /// * `addr` - The node address to connect to
    async fn init(addr: SocketAddr) -> Self;

    /// Get the status of the client
    // fn status(&self) -> ClientStatus;

    /// Find a successor of a given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to find the successor for
    async fn find_successor(&self, id: NodeId) -> Result<Node, ClientError>;

    /// Get the successor of the node
    async fn successor(&self) -> Result<Node, ClientError>;

    /// Get the predecessor of the node
    async fn predecessor(&self) -> Result<Option<Node>, ClientError>;

    /// Notify the node about a new predecessor
    ///
    /// # Arguments
    ///
    /// * `predecessor` - The new predecessor
    async fn notify(&self, predecessor: Node) -> Result<(), ClientError>;

    /// Get the finger table of the node
    ///
    /// # Returns
    ///
    /// A vector of nodes
    async fn get_finger_table(&self) -> Result<Vec<Node>, ClientError>;

    /// Ping the node
    async fn ping(&self) -> Result<(), ClientError>;
}

#[derive(Debug)]
pub enum ClientError {
    ConnectionFailed(Node),
    InvalidRequest(String),
    NotInitialized,
    Unexpected(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::ConnectionFailed(node) => {
                write!(f, "Connection to node {} failed", node.addr())
            }
            ClientError::NotInitialized => write!(f, "Client not initialized"),
            ClientError::Unexpected(message) => write!(f, "{}", message),
            ClientError::InvalidRequest(message) => write!(f, "Invalid request: {}", message),
        }
    }
}

impl From<RecvError> for ClientError {
    fn from(value: RecvError) -> Self {
        log::error!("Error while receiving command result: {}", value);
        ClientError::Unexpected("Error while receiving command result".to_string())
    }
}

// impl <T> From<T> for ClientError
// where
//     T: IntoClientError,
// {
//     fn from(value: T) -> Self {
//         value.into()
//     }
// }

// pub trait IntoClientError {
//     fn into(self) -> ClientError;
// }

pub enum ClientStatus {
    NotConnected,
    Connected,
    Disconnected,
}
