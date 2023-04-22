use chord_rs::{client::ClientError, Node, NodeId};

use crate::{chord_capnp, parser::ParserError};

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
        let node = reply
            .get()
            .unwrap()
            .get_node()
            .unwrap()
            .try_into()
            .map_err(|err: ParserError| err.into());

        sender.send(node).unwrap();
    }

    pub(crate) async fn get_predecessor(
        client: chord_capnp::chord_node::Client,
        sender: CmdResult<Option<Node>>,
    ) {
        let request = client.get_predecessor_request();

        let reply = request.send().promise.await.unwrap(); // TODO: Handle error
        let node = reply.get().unwrap().get_node().unwrap();
        let node = match node.which() {
            Ok(chord_capnp::option::None(())) => Ok(None),
            Ok(chord_capnp::option::Some(Ok(reader))) => {
                let result: Result<Node, ClientError> = reader.try_into().map_err(|err: ParserError| err.into());
                result.map(Some)
            }
            Ok(chord_capnp::option::Some(Err(err))) => map_err(err.into()),
            Err(err) => map_err(err.into()),
        };

        sender.send(node).unwrap();
    }
}

fn map_err<T>(err: capnp::Error) -> Result<T, ClientError> {
    Err(ClientError::Unexpected(format!("{}", err)))
}
impl From<ParserError> for ClientError {
    fn from(err: ParserError) -> Self {
        Self::Unexpected(format!("{}", err))
    }
}
