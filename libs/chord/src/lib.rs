pub mod client;
mod node;
mod service;

use seahash::hash;
use std::net::SocketAddr;

pub use client::Client;
pub use service::NodeService;

pub use service::error;

/// A reference to a node in the chord ring
#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    id: u64,
    addr: SocketAddr,
}

impl Node {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            id: hash(addr.to_string().as_bytes()),
            addr,
        }
    }

    pub fn client<C: Client>(&self) -> C {
        C::init(self.addr)
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn with_id(id: u64, addr: SocketAddr) -> Self {
        Self { id, addr }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns true if the given id is between 2 nodes on a ring
    ///
    /// # Arguments
    ///
    /// * `id` - The id to check
    /// * `node1` - First node id
    /// * `node2` - Second node id
    ///
    /// # Examples
    ///
    /// Check if 10 is between 5 and 15
    ///
    /// ```
    /// use chord_rs::Node;
    ///
    /// let id = 10;
    /// let node1 = 5;
    /// let node2 = 15;
    ///
    /// assert_eq!(Node::is_between_on_ring(id, node1, node2), true);
    /// ```
    ///
    /// Check if 20 is between 15 and 5
    /// ```
    /// use chord_rs::Node;
    ///
    /// let id = 20;
    /// let node1 = 15;
    /// let node2 = 5;
    ///
    /// assert_eq!(Node::is_between_on_ring(id, node1, node2), true);
    /// ```
    pub fn is_between_on_ring(id: u64, node1: u64, node2: u64) -> bool {
        if node1 < node2 {
            node1 < id && id <= node2
        } else {
            node1 < id || id <= node2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_between() {
        assert_eq!(Node::is_between_on_ring(10, 5, 5), true);
        assert_eq!(Node::is_between_on_ring(1, 5, 5), true);
        assert_eq!(Node::is_between_on_ring(10, 5, 1), true);
        assert_eq!(Node::is_between_on_ring(5, 5, 5), true);
        assert_eq!(Node::is_between_on_ring(4, 1, 5), true);
        assert_eq!(Node::is_between_on_ring(5, 1, 5), true);

        assert_eq!(Node::is_between_on_ring(1, 1, 5), false);
        assert_eq!(Node::is_between_on_ring(1, 2, 5), false);

    }
}
