use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use chord_proto::chord_node_server::ChordNode;
pub use chord_proto::chord_node_server::ChordNodeServer;
use chord_proto::{PingRequest, PingResponse};
use chord_rs::{Node, NodeService};
pub use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::client::ChordGrpcClient;

use self::chord_proto::{
    FindSuccessorRequest, FindSuccessorResponse, GetFingerTableRequest, GetFingerTableResponse,
    GetPredecessorRequest, GetPredecessorResponse, NotifyRequest, NotifyResponse,
};

pub mod chord_proto {
    use crate::client::ChordGrpcClient;

    include!(concat!(env!("OUT_DIR"), "/chord.rs"));

    impl Clone for ChordGrpcClient {
        fn clone(&self) -> Self {
            Self {
                client: self.client.clone(),
            }
        }
    }

    unsafe impl Sync for ChordGrpcClient {}
    unsafe impl Send for ChordGrpcClient {}
}

#[derive(Debug, Clone)]
pub struct ChordService {
    node: Arc<NodeService<ChordGrpcClient>>,
}

impl ChordService {
    pub async fn new(addr: SocketAddr, ring: Option<SocketAddr>) -> Self {
        let node_service = Arc::new(NodeService::new(addr));

        if let Some(ring) = ring {
            let node_service = node_service.clone();
            // TODO: make this configurable
            const WAIT_BETWEEN_RETRIES: Duration = Duration::from_secs(3);
            const MAX_RETRIES: u32 = 5;
            let mut attempt = 0;
            loop {
                attempt += 1;
                log::info!("{} attempt to join ring: {:?}", attempt, ring);

                let node = Node::new(ring);
                tokio::time::sleep(Duration::from_secs(1)).await;

                if let Ok(_) = node_service.join(node).await {
                    log::info!("Joined ring: {:?}", ring);
                    break;
                } else {
                    if attempt >= MAX_RETRIES {
                        log::error!("Failed to join ring: {:?}", ring);
                        panic!("Failed to join ring: {:?}", ring)
                    }
                }

                tokio::time::sleep(WAIT_BETWEEN_RETRIES).await;
            }
        }

        let service = node_service.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                // log::info!("Stabilizing...");
                if let Err(err) = service.stabilize().await {
                    log::error!("Stabilize error: {:?}", err);
                }
            }
        });

        // let service = node_service.clone();
        // tokio::spawn(async move {
        //     loop {
        //         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        //         println!("Checking predecessor...");
        //         service.check_predecessor();
        //     }
        // });

        let service = node_service.clone();
        tokio::spawn(async move {
            // TODO: remove this and make it wait for the node to join the ring before starting fixing fingers
            // tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                // log::info!("Fixing fingers...");
                service.fix_fingers().await;
            }
        });

        Self { node: node_service }
    }

    fn map_error(error: chord_rs::error::ServiceError) -> Status {
        match error {
            chord_rs::error::ServiceError::Unexpected(message) => Status::internal(message),
        }
    }
}

pub enum JoinRingError {
    ClientError,
    ServiceError,
}

impl From<chord_rs::error::ServiceError> for JoinRingError {
    fn from(error: chord_rs::error::ServiceError) -> Self {
        match error {
            chord_rs::error::ServiceError::Unexpected(_) => Self::ServiceError,
        }
    }
}

#[tonic::async_trait]
impl ChordNode for ChordService {
    async fn ping(&self, _request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let reply = chord_proto::PingResponse {};

        Ok(Response::new(reply))
    }

    async fn find_successor(
        &self,
        request: Request<FindSuccessorRequest>,
    ) -> Result<Response<FindSuccessorResponse>, Status> {
        let result = self
            .node
            .find_successor(request.get_ref().id.into())
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(result.into()))
    }

    async fn get_predecessor(
        &self,
        _request: Request<GetPredecessorRequest>,
    ) -> Result<Response<GetPredecessorResponse>, Status> {
        let result = self.node.get_predecessor().await.map_err(Self::map_error)?;

        // println!("result: {:?}", result);

        Ok(Response::new(result.into()))
    }

    async fn notify(
        &self,
        request: Request<NotifyRequest>,
    ) -> Result<Response<NotifyResponse>, Status> {
        let node = request.get_ref().node.clone();
        let node = Node::try_from(node.unwrap()).unwrap();

        self.node.notify(node);

        Ok(Response::new(NotifyResponse {}))
    }

    async fn get_finger_table(
        &self,
        _: Request<GetFingerTableRequest>,
    ) -> Result<Response<GetFingerTableResponse>, Status> {
        let finger_table = self.node.finger_table();

        let nodes = finger_table
            .iter()
            .map(|finger| finger.node.clone().into())
            .collect();

        Ok(Response::new(GetFingerTableResponse { nodes }))
    }
}

impl From<chord_rs::Node> for FindSuccessorResponse {
    fn from(node: chord_rs::Node) -> Self {
        FindSuccessorResponse {
            node: Some(node.into()),
        }
    }
}

// impl Into<chord_rs::Node> for chord_proto::Node {
//     fn into(self) -> chord_rs::Node {
//         let ip = self.ip.unwrap();
//         let ip = match ip.version {
//             chord_proto::IpVersion::Ipv4 => IpAddr::V4(ip.address.into()),
//             chord_proto::IpVersion::Ipv6 => IpAddr::V6(ip.address.into()),
//         };

//         let addr = SocketAddr::new(ip, self.port as u16);

//         chord_rs::Node::new(addr)
//     }
// }

impl From<Option<chord_rs::Node>> for GetPredecessorResponse {
    fn from(node: Option<chord_rs::Node>) -> Self {
        GetPredecessorResponse {
            node: node.map(|node| node.into()),
        }
    }
}

impl From<chord_rs::Node> for chord_proto::Node {
    fn from(node: chord_rs::Node) -> Self {
        chord_proto::Node {
            id: node.id().into(),
            ip: Some(node.addr().ip().into()),
            port: node.addr().port() as i32,
        }
    }
}

impl From<IpAddr> for chord_proto::IpAddress {
    fn from(ip: IpAddr) -> Self {
        let (version, address) = match ip {
            IpAddr::V4(v4) => (chord_proto::IpVersion::Ipv4, v4.octets().to_vec()),
            IpAddr::V6(v6) => (chord_proto::IpVersion::Ipv6, v6.octets().to_vec()),
        };

        chord_proto::IpAddress {
            version: version.into(),
            address: address,
        }
    }
}
