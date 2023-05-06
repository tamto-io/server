use std::net::SocketAddr;

use chord_rs::{client::ClientError, Client, Node, NodeId};
use error_stack::{Result, ResultExt, IntoReport};
use thiserror::Error;
use tokio::sync::oneshot::{self, Sender};

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
        self.handle_request(|tx| Command::FindSuccessor(id, tx)).await
        // let (tx, rx) = oneshot::channel();
        // self.spawner.spawn(Command::FindSuccessor(id, tx)).await.unwrap().change_context(ClientError::ConnectionFailed("".to_string()))?;

        // let result = rx.await.into_report().change_context(ClientError::Unexpected("".to_string()))?;
        // result.change_context(ClientError::Unexpected("".to_string()))
    }

    async fn successor(&self) -> Result<Node, ClientError> {
        self.handle_request(|tx| Command::Successor(tx)).await
        // let (tx, rx) = oneshot::channel();
        // self.spawner.spawn(Command::Successor(tx)).await.unwrap().change_context(ClientError::FixMe)?;

        // let result = rx.await.into_report().change_context(ClientError::FixMe)?;
        // result.change_context(ClientError::FixMe)
    }

    async fn successor_list(&self) -> Result<Vec<Node>, ClientError> {
        self.handle_request(|tx| Command::SuccessorList(tx)).await
        // let (tx, rx) = oneshot::channel();
        // self.spawner.spawn(Command::SuccessorList(tx)).await.unwrap().change_context(ClientError::ConnectionFailed("".to_string()))?;

        // let result = rx.await.into_report().change_context(ClientError::Unexpected("".to_string()))?;
        // result.into_report().change_context(ClientError::Unexpected("".to_string()))
    }

    async fn predecessor(&self) -> Result<Option<Node>, ClientError> {
        self.handle_request(|tx| Command::Predecessor(tx)).await
        // let (tx, rx) = oneshot::channel();
        // self.spawner.spawn(Command::Predecessor(tx));

        // let result = rx.await?;
        // Ok(result?)
    }

    async fn notify(&self, predecessor: Node) -> Result<(), ClientError> {
        self.handle_request(|tx| Command::Notify(predecessor, tx)).await
        // let (tx, rx) = oneshot::channel();
        // self.spawner.spawn(Command::Notify(predecessor, tx));

        // let result = rx.await?;
        // Ok(result?)
    }

    async fn ping(&self) -> Result<(), ClientError> {
        self.handle_request(|tx| Command::Ping(tx)).await
        // let (tx, rx) = oneshot::channel();
        // self.spawner.spawn(Command::Ping(tx));

        // let result = rx.await?;
        // Ok(result?)
    }
}

impl ChordCapnpClient {
    async fn handle_request<T>(&self, request: impl FnOnce(Sender<Result<T, CapnpClientError>>) -> Command) -> Result<T, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(request(tx)).await.unwrap().change_context(ClientError::FixMe)?;

        let result = rx.await.into_report().change_context(ClientError::FixMe)?;
        result.change_context(ClientError::FixMe)
    }
}

#[derive(Debug, Error)]
pub(crate) enum CapnpClientError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Ping failed")]
    PingFailed,
    #[error("Find successor failed")]
    FindSuccessorFailed,
    #[error("Get successor failed")]
    GetSuccessorFailed,
    #[error("Get successor list failed")]
    GetSuccessorListFailed,
    #[error("Get predecessor failed")]
    GetPredecessorFailed,
    #[error("Notify failed")]
    NotifyFailed,
}

