use std::sync::{Arc, Mutex};

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
        Self {
            db: Db::new(successor),
        }
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
        let mut state = self.shared_state();
        state.predecessor = Some(predecessor);

        drop(state)
    }

    /// Unset the predecessor of the node
    pub(crate) fn unset_predecessor(&self) {
        let mut state = self.shared_state();
        state.predecessor = None;

        drop(state)
    }

    /// Get the predecessor of the node
    pub(crate) fn predecessor(&self) -> Option<Node> {
        let state = self.shared_state();
        state.predecessor.clone()
    }

    /// Set the successor of the node
    ///
    /// # Arguments
    ///
    /// * `successor` - The successor node
    pub(crate) fn set_successor(&self, successor: Node) {
        let mut state = self.shared_state();
        state.finger_table[0].node = successor;

        drop(state)
    }

    /// Get the successor of the node
    pub(crate) fn successor(&self) -> Node {
        let state = self.shared_state();

        state.finger_table[0].node.clone()
    }

    /// Get the closest preceding node
    /// This is used to find a node that is possibly responsible for a key
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the current node
    /// * `id` - The id of the key we are looking for
    ///
    /// # Returns
    ///
    /// The closest preceding node for the key
    pub(crate) fn closest_preceding_node(&self, node_id: u64, id: u64) -> Option<Node> {
        let state = self.shared_state();

        let fingers = state.finger_table.clone();
        drop(state);

        for finger in fingers.iter().rev() {
            if Node::is_between_on_ring_exclusive(finger.node.id.into(), node_id, id) {
                return Some(finger.node.clone());
            }
        }

        None
    }

    pub(crate) fn update_finger(&self, finger_id: usize, node: Node) {
        let mut state = self.shared_state();
        state.finger_table[finger_id].node = node;

        drop(state);
    }

    pub(crate) fn finger_table(&self) -> Vec<Finger> {
        let state = self.shared_state();
        state.finger_table.clone()
    }

    fn shared_state(&self) -> std::sync::MutexGuard<State> {
        let lock = self.shared.state.lock();
        if let Ok(state) = lock {
            return state;
        } else {
            log::error!("Could not lock state, error: {}", lock.unwrap_err());
            panic!("Could not lock state");
        }
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

    #[test]
    fn test_closest_preceding_node() {
        let node = Node::with_id(NodeId(10), SocketAddr::from(([127, 0, 0, 1], 42001)));
        let store = NodeStore::new(node.clone());
        let successor = Node::with_id(NodeId(20), SocketAddr::from(([127, 0, 0, 1], 42002)));
        let predecessor = Node::with_id(NodeId(1), SocketAddr::from(([127, 0, 0, 1], 42003)));
        store.db().set_predecessor(predecessor.clone());

        store
            .db()
            .finger_table()
            .iter()
            .enumerate()
            .for_each(|(i, finger)| {
                if finger._start < 20 {
                    store.db().update_finger(i, successor.clone());
                } else {
                    store.db().update_finger(i, predecessor.clone());
                }
            });

        assert_eq!(
            store.db().closest_preceding_node(10, 2),
            Some(predecessor.clone())
        );
        assert_eq!(
            store.db().closest_preceding_node(10, 10),
            Some(predecessor.clone())
        );
        assert_eq!(store.db().closest_preceding_node(10, 15), None);
        assert_eq!(
            store.db().closest_preceding_node(10, 21),
            Some(successor.clone())
        );
        assert_eq!(store.db().closest_preceding_node(10, 28), Some(successor));
    }
}
