use std::{fmt::Display, sync::Arc};

use chord_rs_core::{Node, NodeService};

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
        log::trace!("Ping received");
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
        log::trace!("FindSuccessor received");

        let service = self.node.clone();

        ::capnp::capability::Promise::from_future(async move {
            let id = params.get()?.get_id();
            let node = service
                .find_successor(id.into())
                .await
                .map_err(error_parser)?;

            results.insert(node)?;

            Ok(())
        })
    }

    fn get_successor_list(
        &mut self,
        _params: chord_capnp::chord_node::GetSuccessorListParams,
        results: chord_capnp::chord_node::GetSuccessorListResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        log::trace!("GetSuccessorList received");

        let service = self.node.clone();
        ::capnp::capability::Promise::from_future(async move {
            let node = service.get_successor_list().await.map_err(error_parser)?;

            results.insert(node)?;

            Ok(())
        })
    }

    /// Get the predecessor of the node
    ///
    /// # Arguments
    ///
    /// * `_params` - Cap'n'proto message, not used.
    /// * `results` - Cap'n'proto message to write the successor to.
    fn get_predecessor(
        &mut self,
        _params: chord_capnp::chord_node::GetPredecessorParams,
        results: chord_capnp::chord_node::GetPredecessorResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        log::trace!("GetPredecessor received");

        let service = self.node.clone();

        ::capnp::capability::Promise::from_future(async move {
            let maybe_node = service.get_predecessor().await.map_err(error_parser)?;
            results.insert(maybe_node)?;

            Ok(())
        })
    }

    /// Notify the node of a new predecessor
    ///
    /// # Arguments
    ///
    /// * `params` - Cap'n'proto message containing the potential new predecessor.
    /// * `_results` - Cap'n'proto message, not used.
    fn notify(
        &mut self,
        params: chord_capnp::chord_node::NotifyParams,
        _results: chord_capnp::chord_node::NotifyResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        log::trace!("Notify received");

        let service = self.node.clone();

        ::capnp::capability::Promise::from_future(async move {
            let node = params.get()?.get_node()?;
            let node: Node = node.try_into().unwrap(); // TODO: error handling
            service.notify(node);

            Ok(())
        })
    }
}

fn error_parser<T>(err: T) -> capnp::Error
where
    T: Display,
{
    capnp::Error::failed(format!("{}", err))
}
