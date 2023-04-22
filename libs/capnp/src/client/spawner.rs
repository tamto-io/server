use std::net::SocketAddr;

use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;
use tokio::{runtime::Builder, sync::mpsc, task::LocalSet};

use crate::chord_capnp;

#[derive(Clone)]
pub(crate) struct LocalSpawner {
    sender: mpsc::UnboundedSender<super::Command>,
}

impl LocalSpawner {
    pub fn new(addr: SocketAddr) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some(command) = receiver.recv().await {
                    Self::run_local(addr, command).await;
                }
            });

            rt.block_on(local);
        });

        Self { sender }
    }

    pub(crate) fn spawn(&self, task: super::Command) {
        self.sender
            .send(task)
            .expect("Thread with LocalSet has shut down.");
    }

    async fn rpc_system(addr: SocketAddr) -> RpcSystem<rpc_twoparty_capnp::Side> {
        let stream = tokio::net::TcpStream::connect(&addr).await.unwrap();
        stream.set_nodelay(true).unwrap();
        let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));

        return RpcSystem::new(rpc_network, None);
    }

    async fn run_local(addr: SocketAddr, command: super::Command) {
        let mut rpc_system = Self::rpc_system(addr).await;
        let client: chord_capnp::chord_node::Client =
            rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
        tokio::task::spawn_local(rpc_system);

        match command {
            super::Command::Ping(resp) => super::Command::ping(client, resp).await,
            super::command::Command::FindSuccessor(node_id, resp) => {
                super::Command::find_successor(client, node_id, resp).await
            }
            super::command::Command::Predecessor(resp) => {
                super::Command::get_predecessor(client, resp).await
            },
            super::command::Command::Notify(node, resp) => {
                super::Command::notify(client, node, resp).await
            },
            super::command::Command::GetFingerTable(_) => todo!(),
        }
    }
}
