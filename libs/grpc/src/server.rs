use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use chord_proto::chord_node_server::ChordNode;
pub use chord_proto::chord_node_server::ChordNodeServer;
use chord_proto::{PingRequest, PingResponse};
use chord_rs::{Node, NodeService};
use error_stack::Report;
pub use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::client::ChordGrpcClient;

use self::chord_proto::{
    FindSuccessorRequest, FindSuccessorResponse, GetPredecessorRequest, GetPredecessorResponse,
    GetSuccessorResponse, NotifyRequest, NotifyResponse,
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
        const REPLICATION_FACTOR: usize = 3; // TODO: make this configurable
        let node_service = Arc::new(NodeService::new(addr, REPLICATION_FACTOR));

        if let Some(ring) = ring {
            const MAX_RETRIES: u32 = 5;
            chord_rs::server::join_ring(node_service.clone(), ring, MAX_RETRIES).await;
        }
        chord_rs::server::background_tasks(node_service.clone());

        Self { node: node_service }
    }

    fn map_error(error: Report<chord_rs::error::ServiceError>) -> Status {
        let message = error.to_string();
        match error.current_context() {
            chord_rs::error::ServiceError::Unexpected => Status::internal(message),
            chord_rs::error::ServiceError::ClientDisconnected => todo!(),
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
            chord_rs::error::ServiceError::Unexpected => Self::ServiceError,
            chord_rs::error::ServiceError::ClientDisconnected => todo!(),
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

    async fn get_successor(
        &self,
        _request: Request<chord_proto::GetSuccessorRequest>,
    ) -> Result<Response<chord_proto::GetSuccessorResponse>, Status> {
        let result = self.node.get_successor().await.map_err(Self::map_error)?;

        Ok(Response::new(result.into()))
    }

    async fn get_predecessor(
        &self,
        _request: Request<GetPredecessorRequest>,
    ) -> Result<Response<GetPredecessorResponse>, Status> {
        let result = self.node.get_predecessor().await.map_err(Self::map_error)?;

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
}

impl From<chord_rs::Node> for FindSuccessorResponse {
    fn from(node: chord_rs::Node) -> Self {
        FindSuccessorResponse {
            node: Some(node.into()),
        }
    }
}

impl From<chord_rs::Node> for GetSuccessorResponse {
    fn from(node: chord_rs::Node) -> Self {
        GetSuccessorResponse {
            node: Some(node.into()),
        }
    }
}

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
