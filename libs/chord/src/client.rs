use crate::Node;
use async_trait::async_trait;
use mockall::automock;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

#[automock]
#[async_trait]
pub trait Client {
    /// Init the client
    ///
    /// # Arguments
    ///
    /// * `addr` - The node address to connect to
    fn init(addr: SocketAddr) -> Self;

    /// Find a successor of a given id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to find the successor for
    async fn find_successor(&self, id: u64) -> Result<Node, ClientError>;

    /// Get the successor of the node
    fn successor(&self) -> Result<Node, ClientError>;

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
    fn ping(&self) -> Result<(), ClientError>;
}

#[derive(Debug)]
pub enum ClientError {
    ConnectionFailed(Node),
    Unexpected(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::ConnectionFailed(node) => {
                write!(f, "Connection to node {} failed", node.addr())
            }
            ClientError::Unexpected(message) => write!(f, "{}", message),
        }
    }
}
