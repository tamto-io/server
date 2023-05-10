use std::net::SocketAddr;

use chord_core::{client::ClientError, Client, Node, NodeId};
use error_stack::{IntoReport, Result, ResultExt};
use thiserror::Error;
use tokio::sync::oneshot::{self, Sender};

use self::{command::Command, spawner::LocalSpawner};

mod command;
mod spawner;

type CmdResult<T> = oneshot::Sender<Result<T, ClientError>>;

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
        self.handle_request(|tx| Command::FindSuccessor(id, tx))
            .await
    }

    async fn successor(&self) -> Result<Node, ClientError> {
        self.handle_request(|tx| Command::Successor(tx)).await
    }

    async fn successor_list(&self) -> Result<Vec<Node>, ClientError> {
        self.handle_request(|tx| Command::SuccessorList(tx)).await
    }

    async fn predecessor(&self) -> Result<Option<Node>, ClientError> {
        self.handle_request(|tx| Command::Predecessor(tx)).await
    }

    async fn notify(&self, predecessor: Node) -> Result<(), ClientError> {
        self.handle_request(|tx| Command::Notify(predecessor, tx))
            .await
    }

    async fn ping(&self) -> Result<(), ClientError> {
        self.handle_request(|tx| Command::Ping(tx)).await
    }
}

impl ChordCapnpClient {
    async fn handle_request<T>(
        &self,
        request: impl FnOnce(Sender<Result<T, ClientError>>) -> Command,
    ) -> Result<T, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(request(tx)).await.unwrap()?;

        let result = rx
            .await
            .into_report()
            .change_context(ClientError::Unexpected)?;
        result
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
}
