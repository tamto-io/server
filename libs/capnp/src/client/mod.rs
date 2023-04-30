use std::net::SocketAddr;

use chord_rs::{client::ClientError, Client, Node, NodeId};
use tokio::sync::oneshot;

use self::{command::Command, spawner::LocalSpawner};

mod command;
mod spawner;

type CmdResult<T> = oneshot::Sender<Result<T, CapnpClientError>>;

#[derive(Clone)]
pub struct ChordCapnpClient {
    spawner: LocalSpawner,
}

#[async_trait::async_trait]
impl Client for ChordCapnpClient {
    async fn init(addr: SocketAddr) -> Self {
        let spawner = LocalSpawner::new(addr);

        Self { spawner }
    }

    async fn find_successor(&self, id: NodeId) -> Result<Node, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::FindSuccessor(id, tx));

        let result = rx.await?;
        Ok(result?)
    }

    async fn successor(&self) -> Result<Node, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Successor(tx));

        let result = rx.await?;
        Ok(result?)
    }

    async fn successor_list(&self) -> Result<Vec<Node>, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::SuccessorList(tx));

        let result = rx.await?;
        Ok(result?)
    }

    async fn predecessor(&self) -> Result<Option<Node>, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Predecessor(tx));

        let result = rx.await?;
        Ok(result?)
    }

    async fn notify(&self, predecessor: Node) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Notify(predecessor, tx));

        let result = rx.await?;
        Ok(result?)
    }

    async fn ping(&self) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Ping(tx));

        let result = rx.await?;
        Ok(result?)
    }
}

#[derive(Debug)]
pub(crate) enum CapnpClientError {
    InvalidRequest(String),
    ConnectionFailed(String),
    Unexpected(String),
}
