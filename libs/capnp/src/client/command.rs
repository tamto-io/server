use chord_rs::{NodeId, Node};

use crate::chord_capnp;

use super::CmdResult;

#[derive(Debug)]
pub(crate) enum Command {
    FindSuccessor(NodeId, CmdResult<Node>),
    Predecessor(CmdResult<Option<Node>>),
    Notify(Node, CmdResult<()>),
    GetFingerTable(CmdResult<Vec<Node>>),
    Ping(CmdResult<()>),
}

impl Command {
    pub(crate) async fn ping(client: chord_capnp::chord_node::Client, sender: CmdResult<()>) {
        let request = client.ping_request();

        let _ = request.send().promise.await;

        sender.send(Ok(())).unwrap();
    }

    pub(crate) async fn find_successor(
        client: chord_capnp::chord_node::Client,
        id: NodeId,
        sender: CmdResult<Node>,
    ) {
        let mut request = client.find_successor_request();
        request.get().set_id(id.into());

        let reply = request.send().promise.await.unwrap(); // TODO: Handle error
        let node = reply.get().unwrap().get_node().unwrap().try_into();

        sender.send(node).unwrap();

    }
}
