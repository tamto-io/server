use std::net::SocketAddr;

use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use chord_core::client::ClientError;
use error_stack::{IntoReport, Report, ResultExt};
use futures::AsyncReadExt;
use thiserror::Error;
use tokio::{
    runtime::Builder,
    sync::{mpsc, oneshot},
    task::LocalSet,
};

use crate::chord_capnp;

use super::command::Command;

#[derive(Clone)]
pub(crate) struct LocalSpawner {
    sender: mpsc::UnboundedSender<(
        super::Command,
        oneshot::Sender<Result<(), Report<ClientError>>>,
    )>,
}

impl LocalSpawner {
    pub fn new(addr: SocketAddr) -> Self {
        let (sender, mut receiver) =
            mpsc::unbounded_channel::<(Command, oneshot::Sender<Result<(), Report<ClientError>>>)>(
            );
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some((command, result_sender)) = receiver.recv().await {
                    let context = command.get_error();
                    if let Err(report) = Self::run_local(addr, command).await {
                        match report.current_context() {
                            SpawnerError::ClientConnectionError => {
                                log::debug!("{report:?}");
                            }
                            _ => {
                                log::error!("Error when handling a request: {report:?}");
                            }
                        }
                        let _ = result_sender.send(Err(report.change_context(context)));
                    } else {
                        let _ = result_sender.send(Ok(()));
                    };
                }
            });

            rt.block_on(local);
        });

        Self { sender }
    }

    pub(crate) fn spawn(
        &self,
        task: super::Command,
    ) -> oneshot::Receiver<Result<(), Report<ClientError>>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send((task, tx))
            .expect("Thread with LocalSet has shut down.");

        rx
    }

    async fn rpc_system(
        addr: SocketAddr,
    ) -> Result<RpcSystem<rpc_twoparty_capnp::Side>, SpawnerError> {
        let stream = tokio::net::TcpStream::connect(&addr).await?;

        stream.set_nodelay(true)?;
        let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));

        return Ok(RpcSystem::new(rpc_network, None));
    }

    async fn run_local(
        addr: SocketAddr,
        command: super::Command,
    ) -> Result<(), Report<SpawnerError>> {
        let mut rpc_system = Self::rpc_system(addr)
            .await
            .into_report()
            .attach_printable_lazy(|| format!("Client address: {:?}", addr))?;
        let client: chord_capnp::chord_node::Client =
            rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
        let disconnector = rpc_system.get_disconnector();
        tokio::task::spawn_local(rpc_system);

        match command {
            super::command::Command::FindSuccessor(node_id, resp) => {
                super::Command::find_successor(client, node_id, resp).await
            }
            super::command::Command::Predecessor(resp) => {
                super::Command::get_predecessor(client, resp).await
            }
            super::command::Command::Notify(node, resp) => {
                super::Command::notify(client, node, resp).await
            }
            super::command::Command::Successor(resp) => {
                super::Command::get_successor(client, resp).await
            }
            super::command::Command::SuccessorList(resp) => {
                super::Command::get_successor_list(client, resp).await
            }
            super::Command::Ping(resp) => super::Command::ping(client, resp).await,
        }

        if let Err(err) = disconnector.await {
            log::error!("Error disconnecting: {:?}", err);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub(crate) enum SpawnerError {
    #[error("Failed to connect to client")]
    ClientConnectionError,

    #[error("Other error: {0:?}")]
    Other(std::io::Error),
}

impl From<std::io::Error> for SpawnerError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::NotConnected
            | std::io::ErrorKind::AddrNotAvailable
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::Interrupted => Self::ClientConnectionError,
            _ => Self::Other(err),
        }
    }
}
