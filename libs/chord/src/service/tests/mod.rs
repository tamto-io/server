use crate::client::__mock_MockClient_Client::{__ping, __find_successor, __predecessor, __successor_list};
use crate::client::{ClientsPool, MockClient, self};
use crate::{Node, NodeId, NodeService};
use std::net::SocketAddr;

mod check_predecessor;
mod find_successor;
mod fix_fingers;
mod join;
mod notify;
mod reconcile_successors;
mod stabilize;

use crate::node::store::NodeStore;
use crate::node::Finger;
use error_stack::Report;
use lazy_static::lazy_static;
use mockall::predicate;
use std::sync::{Mutex, MutexGuard};

lazy_static! {
    pub(crate) static ref MTX: Mutex<()> = Mutex::new(());
}

// When a test panics, it will poison the Mutex. Since we don't actually
// care about the state of the data we ignore that it is poisoned and grab
// the lock regardless.  If you just do `let _m = &MTX.lock().unwrap()`, one
// test panicking will cause all other tests that try and acquire a lock on
// that Mutex to also panic.
pub(crate) fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
    match m.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn node(id: u64) -> Node {
    let addr = SocketAddr::from(([127, 0, 0, 1], 42000 + id as u16));
    Node::with_id(id, addr)
}

impl Default for NodeService<MockClient> {
    fn default() -> Self {
        let node = Node::with_id(8, SocketAddr::from(([127, 0, 0, 1], 42001)));
        let store = NodeStore::new(node.clone(), 3);
        Self {
            id: node.id,
            addr: node.addr,
            store,
            clients: ClientsPool::default(),
        }
    }
}

impl NodeService<MockClient> {
    fn test_service(id: u64) -> Self {
        let node = Node::with_id(id, SocketAddr::from(([127, 0, 0, 1], 42000 + id as u16)));
        let store = NodeStore::new(node.clone(), 3);
        Self {
            id: node.id,
            addr: node.addr,
            store,
            clients: ClientsPool::default(),
        }
    }

    fn find_closest_successor(id: NodeId, nodes: &Vec<Node>) -> Node {
        let mut nodes = nodes.clone();
        nodes.sort_by(|b, a| a.id.cmp(&b.id));

        let smallest = nodes.last().unwrap().clone();
        let mut closest = nodes[0].clone();
        for node in nodes {
            if node.id == id {
                return node;
            }
            if node.id < closest.id && node.id > id {
                closest = node;
            } else if node.id < id && Node::is_between_on_ring(id.0, closest.id.0, node.id.0) {
                closest = node;
            }
        }

        if closest.id > id {
            closest
        } else {
            smallest
        }
    }

    pub(crate) fn with_fingers(&mut self, nodes_ids: Vec<u64>) {
        self.with_fingers_sized(64, nodes_ids);
    }

    pub(crate) fn with_fingers_sized(&mut self, size: u8, nodes_ids: Vec<u64>) {
        let mut nodes: Vec<Node> = nodes_ids.into_iter().map(|id| node(id)).collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        for i in 1..size + 1 {
            let finger_id = Finger::sized_finger_id(size, self.id.0, (i) as u8);

            let closest = Self::find_closest_successor(NodeId(finger_id), &nodes);
            self.store.db().update_finger((i - 1) as usize, closest);
        }
    }

    // pub(crate) fn collect_finger_ids(&self) -> Vec<u64> {
    //     self.store.db().finger_table().iter().map(|f| f._start).collect()
    // }

    pub(crate) fn collect_finger_node_ids(&self) -> Vec<u64> {
        self.store
            .db()
            .finger_table()
            .iter()
            .map(|f| f.node.id.0)
            .collect()
    }
}

impl MockClient {
    /// Mock find_successor method.
    ///
    /// # Arguments
    ///
    /// * `id` - The id for which to find the successor.
    /// * `return_node` - The successor to return.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::net::SocketAddr;
    /// use crate::client::MockClient;
    /// use crate::service::tests::{get_lock, MTX};
    ///
    /// let _m = get_lock(&MTX);
    /// let ctx = MockClient::init_context();
    ///
    /// ctx.expect().returning(|addr: SocketAddr| {
    ///     let mut client = MockClient::new();
    ///     // Node with port 42014 will respond with 21 as a successor for id 16.
    ///     if addr.port() == 42014 { client.mock_find_successor(16, 21); }
    ///
    ///     client
    /// });
    /// ```
    fn mock_find_successor(&mut self, id: NodeId, return_node: u64) {
        self.expect_find_successor()
            .with(predicate::eq(id))
            .times(1)
            .returning(move |_| Ok(node(return_node)));
    }
}

trait ExpectationExt<E> {
    fn returning_error(&mut self, err: E) -> &mut Self;
}


impl ExpectationExt<client::ClientError> for __ping::Expectation {
    fn returning_error(&mut self, err: client::ClientError) -> &mut Self {
        self.returning(move || {
            Err(Report::new(err.to_owned()))
        })
    }
}

impl ExpectationExt<client::ClientError> for __find_successor::Expectation {
    fn returning_error(&mut self, err: client::ClientError) -> &mut Self {
        self.returning(move |_| {
            Err(Report::new(err.to_owned()))
        })
    }
}

impl ExpectationExt<client::ClientError> for __predecessor::Expectation {
    fn returning_error(&mut self, err: client::ClientError) -> &mut Self {
        self.returning(move || {
            Err(Report::new(err.to_owned()))
        })
    }
}

impl ExpectationExt<client::ClientError> for __successor_list::Expectation {
    fn returning_error(&mut self, err: client::ClientError) -> &mut Self {
        self.returning(move || {
            Err(Report::new(err.to_owned()))
        })
    }
}

impl MockClient {
    pub fn mock(addr: SocketAddr, node_id: u64, mock_fn: impl FnOnce(MockClient) -> MockClient) -> Self {
        let mut client = MockClient::new();

        if addr.port() == 42000 + node_id as u16 {
            client = mock_fn(client);
        }

        client
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_finger_table() {
        let mut service = NodeService::default();
        let nodes = vec![1, 16, 32, 64];
        service.with_fingers(nodes.clone());

        assert_eq!(9, service.store.db().finger_table()[0]._start);
        assert_eq!(16, service.store.db().finger_table()[0].node.id.0);
        assert_eq!(10, service.store.db().finger_table()[1]._start);
        assert_eq!(16, service.store.db().finger_table()[1].node.id.0);
        assert_eq!(12, service.store.db().finger_table()[2]._start);
        assert_eq!(16, service.store.db().finger_table()[2].node.id.0);
        assert_eq!(16, service.store.db().finger_table()[3]._start);
        assert_eq!(16, service.store.db().finger_table()[3].node.id.0);

        assert_eq!(264, service.store.db().finger_table()[8]._start);
        assert_eq!(1, service.store.db().finger_table()[8].node.id.0);

        service.id = NodeId(2);
        service.with_fingers(nodes.clone());

        assert_eq!(16, service.store.db().finger_table()[0].node.id.0);
        assert_eq!(16, service.store.db().finger_table()[3].node.id.0);
        assert_eq!(32, service.store.db().finger_table()[4].node.id.0);
        assert_eq!(64, service.store.db().finger_table()[5].node.id.0);
        assert_eq!(1, service.store.db().finger_table()[6].node.id.0);
        assert_eq!(1, service.store.db().finger_table()[63].node.id.0);

        service.id = NodeId(154);
        service.with_fingers(nodes.clone());

        assert_eq!(1, service.store.db().finger_table()[0].node.id.0);
        assert_eq!(1, service.store.db().finger_table()[63].node.id.0);

        service.id = NodeId(u64::MAX - 1);
        service.with_fingers(nodes.clone());

        assert_eq!(1, service.store.db().finger_table()[0].node.id.0);
        assert_eq!(1, service.store.db().finger_table()[1].node.id.0);
        assert_eq!(12, service.store.db().finger_table()[2]._start);
        assert_eq!(16, service.store.db().finger_table()[2].node.id.0);
        assert_eq!(24, service.store.db().finger_table()[4]._start);
        assert_eq!(16, service.store.db().finger_table()[4].node.id.0);

        // service.id = NodeId(1);
        // service.with_fingers_sized(6, nodes.clone());
        // assert_eq!(6, service.store.db().finger_table().len());

        // assert_eq!(16, service.store.db().finger_table()[0].node.id.0);
        // assert_eq!(16, service.store.db().finger_table()[1].node.id.0);
        // assert_eq!(5, service.store.db().finger_table()[2]._start);
        // assert_eq!(16, service.store.db().finger_table()[2].node.id.0);
        // assert_eq!(17, service.store.db().finger_table()[4]._start);
        // assert_eq!(32, service.store.db().finger_table()[4].node.id.0);
    }

    #[test]
    fn test_closest_successor() {
        let nodes = vec![node(1), node(16), node(32), node(64)];

        let closest = NodeService::find_closest_successor(NodeId(1), &nodes);
        assert_eq!(NodeId(1), closest.id);

        let closest = NodeService::find_closest_successor(NodeId(2), &nodes);
        assert_eq!(NodeId(16), closest.id);

        let closest = NodeService::find_closest_successor(NodeId(25), &nodes);
        assert_eq!(NodeId(32), closest.id);

        let closest = NodeService::find_closest_successor(NodeId(33), &nodes);
        assert_eq!(NodeId(64), closest.id);

        let closest = NodeService::find_closest_successor(NodeId(64), &nodes);
        assert_eq!(NodeId(64), closest.id);

        let closest = NodeService::find_closest_successor(NodeId(65), &nodes);
        assert_eq!(NodeId(1), closest.id);
    }
}
