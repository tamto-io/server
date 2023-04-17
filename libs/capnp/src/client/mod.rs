use std::net::SocketAddr;

use chord_rs::{Client, client::ClientError, NodeId, Node};
use tokio::sync::oneshot;

use self::{spawner::LocalSpawner, command::Command};

mod spawner;
mod command;

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
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::FindSuccessor(id, tx));

        rx.await.unwrap()
    }

    async fn successor(&self) -> Result<Node, ClientError> {
        self.get_finger_table().await.map(|table| table[0].clone())
    }

    async fn predecessor(&self) -> Result<Option<Node>, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Predecessor(tx));

        rx.await.unwrap()
    }

    async fn notify(&self, predecessor: Node) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Notify(predecessor, tx));

        rx.await.unwrap()
    }

    async fn get_finger_table(&self) -> Result<Vec<Node>, ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::GetFingerTable(tx));

        rx.await.unwrap()
    }

    async fn ping(&self) -> Result<(), ClientError> {
        let (tx, rx) = oneshot::channel();
        self.spawner.spawn(Command::Ping(tx));

        rx.await.unwrap()
    }
}
