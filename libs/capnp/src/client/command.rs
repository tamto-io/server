use chord_rs::{Node, NodeId};
use futures::Future;

use crate::{
    chord_capnp::{self, chord_node::Client},
    client::CapnpClientError,
    parser::{ParserError, ResultBuilder},
};

use super::CmdResult;

#[derive(Debug)]
pub(crate) enum Command {
    FindSuccessor(NodeId, CmdResult<Node>),
    Successor(CmdResult<Node>),
    Predecessor(CmdResult<Option<Node>>),
    Notify(Node, CmdResult<()>),
    Ping(CmdResult<()>),
}

impl Command {
    pub(crate) async fn ping(client: Client, sender: CmdResult<()>) {
        handle_request(sender, || async {
            let request = client.ping_request();

            request.send().promise.await?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn find_successor(client: Client, id: NodeId, sender: CmdResult<Node>) {
        handle_request(sender, || async {
            let mut request = client.find_successor_request();
            request.get().set_id(id.into());

            let reply = request.send().promise.await?;
            let node = reply.get()?.get_node()?.try_into()?;

            Ok(node)
        })
        .await
    }

    pub(crate) async fn get_successor(client: Client, sender: CmdResult<Node>) {
        handle_request(sender, || async {
            let request = client.get_successor_request();

            let reply = request.send().promise.await?;
            let successor = reply.get()?.get_node()?.try_into()?;
            Ok(successor)
        })
        .await;
    }

    pub(crate) async fn get_predecessor(client: Client, sender: CmdResult<Option<Node>>) {
        handle_request(sender, || async {
            let request = client.get_predecessor_request();

            let reply = request.send().promise.await?;
            let node = reply.get()?.get_node()?;
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
        })
        .await
    }

    pub(crate) async fn notify(client: Client, predecessor: Node, sender: CmdResult<()>) {
        handle_request(sender, || async {
            let mut request = client.notify_request();
            let node = request.get().init_node();
            node.insert(predecessor)?;

            let _ = request.send().promise.await;
            Ok(())
        })
        .await;
    }
}

async fn handle_request<F, Res>(sender: CmdResult<Res>, f: impl FnOnce() -> F)
where
    F: Future<Output = Result<Res, CapnpClientError>>,
    Res: std::fmt::Debug,
{
    let result = f().await;

    sender.send(result).unwrap();
}
