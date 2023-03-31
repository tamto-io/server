use std::sync::{Mutex, Arc};

use crate::node::Finger;
use crate::Node;

/// A node in the chord ring
///
/// This struct is used to represent a node in the chord ring.
#[derive(Debug)]
pub struct NodeStore {
    db: Db,
}
#[derive(Debug, Clone)]
pub(crate) struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    predecessor: Option<Node>,
    finger_table: Vec<Finger>,
}

impl NodeStore {
    /// Create a new node store
    ///
    /// # Arguments
    ///
    /// * `successor` - The immediate successor of the current node
    pub(crate) fn new(successor: Node) -> Self {
        Self { db: Db::new(successor) }
    }

    /// Get the shared database. Internally, this is an
    /// `Arc`, so a clone only increments the ref count.
    pub(crate) fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    pub(crate) fn new(node: Node) -> Db {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                predecessor: None,
                finger_table: Finger::init_finger_table(node),
            }),
            // background_task: Notify::new(),
        });

        // Start the background task.
        // tokio::spawn(purge_expired_tasks(shared.clone()));

        Db { shared }
    }


    /// Set the predecessor of the node
    ///
    /// # Arguments
    ///
    /// * `predecessor` - The predecessor node
    pub(crate) fn set_predecessor(&self, predecessor: Node) {
        let mut state = self.shared.state.lock().unwrap();
        state.predecessor = Some(predecessor);

        drop(state)
    }

    /// Unset the predecessor of the node
    pub(crate) fn unset_predecessor(&self) {
        let mut state = self.shared.state.lock().unwrap();
        state.predecessor = None;

        drop(state)
    }

    /// Get the predecessor of the node
    pub(crate) fn predecessor(&self) -> Option<Node> {
        let state = self.shared.state.lock().unwrap();
        state.predecessor.clone()
    }

    /// Set the successor of the node
    ///
    /// # Arguments
    ///
    /// * `successor` - The successor node
    pub(crate) fn set_successor(&self, successor: Node) {
        let mut state = self.shared.state.lock().unwrap();
        state.finger_table[0].node = successor;

        drop(state)
    }

    /// Get the successor of the node
    pub(crate) fn successor(&self) -> Node {
        let state = self.shared.state.lock().unwrap();

        state.finger_table[0].node.clone()
    }

    /// TODO: Make sure the lock is dropped
    ///       What happens if the finger is returned in the loop? Is the lock dropped?
    pub(crate) fn closest_preceding_node(&self, node_id: u64, id: u64) -> Node {
        let state = self.shared.state.lock().unwrap();

        for finger in state.finger_table.iter().rev() {
            if finger.start > node_id && finger.node.id.0 < id && finger.start < id {
                return finger.node.clone();
            } else if id < node_id {
                // if the id is smaller than the current node, we return the last finger
                return finger.node.clone();
            }
        }

        drop(state);

        self.successor()
    }

    pub(crate) fn update_finger(&self, finger_id: usize, node: Node) {
        let mut state = self.shared.state.lock().unwrap();
        state.finger_table[finger_id].node = node;

        drop(state);
    }

    pub(crate) fn finger_table(&self) -> Vec<Finger> {
        let state = self.shared.state.lock().unwrap();
        state.finger_table.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::NodeId;

    use super::*;
    use std::net::SocketAddr;

    #[test]
    fn test_new() {
        let node = Node::with_id(NodeId(1), SocketAddr::from(([127, 0, 0, 1], 42001)));
        let store = NodeStore::new(node.clone());
        let store = store.db();

        assert_eq!(store.successor(), node);
        assert_eq!(store.predecessor(), None);
    }

    #[test]
    fn test_predecessor() {
        let node = Node::with_id(NodeId(1), SocketAddr::from(([127, 0, 0, 1], 42001)));
        let store = NodeStore::new(node.clone());
        let predecessor = Node::with_id(NodeId(2), SocketAddr::from(([127, 0, 0, 1], 42002)));
        assert_eq!(store.db().predecessor(), None);
        store.db().set_predecessor(predecessor.clone());

        assert_eq!(store.db().predecessor(), Some(predecessor));

        store.db().unset_predecessor();
        assert_eq!(store.db().predecessor(), None);
    }

    #[test]
    fn test_successor() {
        let node = Node::with_id(NodeId(1), SocketAddr::from(([127, 0, 0, 1], 42001)));
        let store = NodeStore::new(node.clone());
        let successor = Node::with_id(NodeId(2), SocketAddr::from(([127, 0, 0, 1], 42002)));
        assert_eq!(store.db().successor(), node);
        store.db().set_successor(successor.clone());

        assert_eq!(store.db().successor(), successor);
    }
}
