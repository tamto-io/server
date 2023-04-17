use std::sync::Arc;

use chord_rs::NodeService;

use crate::{chord_capnp, parser::ResultBuilder};

use super::client::ChordCapnpClient;

/// Implementation of the chord_node interface
pub(crate) struct NodeServerImpl {
    node: Arc<NodeService<ChordCapnpClient>>,
}

impl NodeServerImpl {
    /// Create a new instance of the Cap'n'proto server
    ///
    /// # Arguments
    ///
    /// * `node` - The Chord node service.
    pub fn new(node: Arc<NodeService<ChordCapnpClient>>) -> Self {
        Self { node }
    }
}

impl chord_capnp::chord_node::Server for NodeServerImpl {
    /// Ping the node
    ///
    /// Just responds with an empty message.
    fn ping(
        &mut self,
        _params: chord_capnp::chord_node::PingParams,
        mut _results: chord_capnp::chord_node::PingResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        log::info!("Ping received");
        ::capnp::capability::Promise::ok(())
    }

    /// Find the successor of a given id
    ///
    /// # Arguments
    ///
    /// * `params` - Cap'n'proto message containing the id to find the successor of.
    /// * `results` - Cap'n'proto message to write the successor to.
    fn find_successor(
        &mut self,
        params: chord_capnp::chord_node::FindSuccessorParams,
        results: chord_capnp::chord_node::FindSuccessorResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        log::info!("FindSuccessor received");

        let service = self.node.clone();
        let id = params.get().unwrap().get_id();

        ::capnp::capability::Promise::from_future(async move {
            let node = service.find_successor(id.into()).await.unwrap();

            results.insert(node).unwrap();

            Ok(())
        })
    }
}
