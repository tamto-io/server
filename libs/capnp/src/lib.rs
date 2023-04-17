use std::{net::{SocketAddr, IpAddr}, sync::Arc};

use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use chord_rs::NodeService;
use client::ChordCapnpClient;
use futures::{AsyncReadExt, TryFutureExt};

pub mod client;
pub mod parser;

pub mod chord_capnp {
    use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4, Ipv6Addr, SocketAddrV6, IpAddr};

    use chord_rs::{Node, client::ClientError};

    use self::chord_node::node::ip_address;

    include!(concat!(env!("OUT_DIR"), "/capnp/chord_capnp.rs"));

}

struct NodeServerImpl {
    node: Arc<NodeService<ChordCapnpClient>>,
}

impl chord_capnp::chord_node::Server for NodeServerImpl {
    fn ping(&mut self, _params: chord_capnp::chord_node::PingParams, mut _results: chord_capnp::chord_node::PingResults) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        log::info!("Ping received");
        ::capnp::capability::Promise::ok(())
    }

    fn find_successor(&mut self, params: chord_capnp::chord_node::FindSuccessorParams<>, mut results: chord_capnp::chord_node::FindSuccessorResults<>) ->  capnp::capability::Promise<(), capnp::Error> {
        log::info!("FindSuccessor received");

        let service = self.node.clone();
        let id = params.get().unwrap().get_id();

        ::capnp::capability::Promise::from_future(async move {
            let node = service.find_successor(id.into()).await.unwrap();

            let mut node_result = results.get().init_node();
            node_result.set_id(node.id().into());

            let mut address = node_result.init_address();
            address.set_port(node.addr().port());

            match node.addr().ip() {
                IpAddr::V4(v4) => {
                    let octets: Vec<u8> = v4.octets().to_vec();
                    let mut ip = address.init_ipv4(4);
                    for i in 0..4 {
                        ip.set(i, octets[i as usize]);
                    }
                },
                IpAddr::V6(v6) => {
                    let segments: Vec<u16> = v6.segments().to_vec();
                    let mut ip = address.init_ipv6(8);
                    for i in 0..8 {
                        ip.set(i, segments[i as usize]);
                    }
                }
            }

            Ok(())
        })
    }
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
                let server = NodeServerImpl {
                    node: self.node.clone(),
                };
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
