use std::{net::SocketAddr, sync::Arc};

use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use chord_rs::NodeService;
use client::ChordCapnpClient;
use futures::AsyncReadExt;

pub mod client;

pub mod chord_capnp {
    use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4, Ipv6Addr, SocketAddrV6, IpAddr};

    use chord_rs::{Node, client::ClientError};

    use self::chord_node::node::ip_address;

    include!(concat!(env!("OUT_DIR"), "/capnp/chord_capnp.rs"));

    impl TryFrom<chord_node::node::Reader<'_>> for Node {
        // fn from(node: chord_node::node::Reader<'_>) -> Self {
    
        //     Self {
        //         id: node.get_id().into(),
        //         addr: node.get_address().unwrap().into(),
        //     }
        // }

        type Error = ClientError;

        fn try_from(value: chord_node::node::Reader<'_>) -> Result<Self, Self::Error> {
            todo!()
        }
    }

    impl From<ip_address::Reader<'_>> for SocketAddr {
        
        fn from(addr: ip_address::Reader<'_>) -> Self {
            let port = addr.get_port();
            let address = match addr.which().unwrap() {
                ip_address::Which::Ipv4(ipv4) => {
                    let ip = ipv4.unwrap();
                    let mut array = [0; 4];
                    array.copy_from_slice(&ip.as_slice().unwrap());
                    let ip = IpAddr::V4(Ipv4Addr::from(array));
                    SocketAddr::new(ip, port)
                }
                ip_address::Which::Ipv6(ipv6) => {
                    let ip = ipv6.unwrap();
                    let mut array = [0; 8];
                    array.copy_from_slice(&ip.as_slice().unwrap());
                    let ip = IpAddr::V6(Ipv6Addr::from(array));
                    
                    SocketAddr::new(ip, port)
                }
            };

            address
        }
    }
}

struct NodeServerImpl;

impl chord_capnp::chord_node::Server for NodeServerImpl {
    fn ping(&mut self, _params: chord_capnp::chord_node::PingParams, mut _results: chord_capnp::chord_node::PingResults) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        log::info!("Ping received");
        ::capnp::capability::Promise::ok(())
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
                let listener = tokio::net::TcpListener::bind(&self.addr).await.unwrap();
                let chord_node_client: chord_capnp::chord_node::Client = capnp_rpc::new_client(NodeServerImpl);

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
