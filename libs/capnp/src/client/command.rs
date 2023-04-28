use chord_rs::{Node, NodeId};

use crate::{chord_capnp, parser::{ParserError, ResultBuilder}, client::CapnpClientError};

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
        async fn get_predecessor_impl(client: chord_capnp::chord_node::Client) -> Result<Option<Node>, CapnpClientError> {
            let request = client.get_predecessor_request();

            let reply = request.send().promise.await?;
            let node = reply.get().unwrap().get_node().unwrap();
            match node.which() {
                Ok(chord_capnp::option::None(())) => Ok(None),
                Ok(chord_capnp::option::Some(Ok(reader))) => {
                    let result: Result<Node, ParserError> = reader.try_into();
                    let node = result?;
                    Ok(Some(node))
                }
                Ok(chord_capnp::option::Some(Err(err))) => Err(err.into()),
                Err(err) => Err(err.into()),
            }
        }

        let node = get_predecessor_impl(client).await;

        sender.send(node).unwrap();
    }

    pub(crate) async fn notify(
        client: chord_capnp::chord_node::Client,
        predecessor: Node,
        sender: CmdResult<()>,
    ) {
        async fn notify_impl(client: chord_capnp::chord_node::Client, predecessor: Node) -> Result<(), CapnpClientError> {
            let mut request = client.notify_request();
            let node = request.get().init_node();
            node.insert(predecessor)?;

            let _ = request.send().promise.await;
            Ok(())
        }

        let result = notify_impl(client, predecessor).await;

        sender.send(result).unwrap();
    }

    // pub(crate) async fn get_finger_table(
    //     client: chord_capnp::chord_node::Client,
    //     sender: CmdResult<Vec<Node>>,
    // ) {
    //     let request = client.get_finger_table_request();

    //     let reply = request.send().promise.await.unwrap(); // TODO: Handle error
    //     let table = reply.get().unwrap().get_table().unwrap();
    //     let table = table.iter().map(|node| node.unwrap().try_into()).collect::<Result<Vec<Node>, ParserError>>()
    //         .map_err(|err: ParserError| err.into());

    //     sender.send(table).unwrap();
    // }
}
