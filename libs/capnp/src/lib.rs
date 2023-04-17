use std::{net::{SocketAddr, IpAddr}, sync::Arc};

use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use chord_rs::NodeService;
use client::ChordCapnpClient;
use futures::{AsyncReadExt, TryFutureExt};

pub mod client;
pub mod parser;
mod server;

pub mod chord_capnp {
    use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4, Ipv6Addr, SocketAddrV6, IpAddr};

    use chord_rs::{Node, client::ClientError};

    use self::chord_node::node::ip_address;

    include!(concat!(env!("OUT_DIR"), "/capnp/chord_capnp.rs"));

}

pub struct Server {
    addr: SocketAddr,
    node: Arc<NodeService<ChordCapnpClient>>,
}

impl Server {
    pub async fn new(addr: SocketAddr, ring: Option<SocketAddr>) -> Self {
        let node_service = Arc::new(NodeService::new(addr));

        Self { addr, node: node_service }
    }

    pub async fn run(&self) {
        tokio::task::LocalSet::new()
            .run_until(async move {
                let server = server::NodeServerImpl::new(self.node.clone());
                let listener = tokio::net::TcpListener::bind(&self.addr).await.unwrap();
                let chord_node_client: chord_capnp::chord_node::Client = capnp_rpc::new_client(server);

                loop {
                    let (stream, _) = listener.accept().await.unwrap();
                    stream.set_nodelay(true).unwrap();
                    let (reader, writer) =
                        tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                    let network = twoparty::VatNetwork::new(
                        reader,
                        writer,
                        rpc_twoparty_capnp::Side::Server,
                        Default::default(),
                    );

                    let rpc_system =
                        RpcSystem::new(Box::new(network), Some(chord_node_client.clone().client));

                    tokio::task::spawn_local(rpc_system);
                }
            })
        .await
    }
}
