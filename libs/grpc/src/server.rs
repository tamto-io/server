use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    thread,
};

use chord_proto::chord_node_server::ChordNode;
pub use chord_proto::chord_node_server::ChordNodeServer;
use chord_proto::{PingRequest, PingResponse};
use chord_rs::NodeService;
use tokio::task::JoinHandle;
pub use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::client::ChordGrpcClient;

use self::chord_proto::{FindSuccessorRequest, FindSuccessorResponse};

pub mod chord_proto {
    include!(concat!(env!("OUT_DIR"), "/chord.rs"));
}

#[derive(Debug)]
pub struct ChordService {
    node: NodeService<ChordGrpcClient>,
}

impl ChordService {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            node: NodeService::new(addr),
        }
    }

    fn map_error(error: chord_rs::error::ServiceError) -> Status {
        match error {
            chord_rs::error::ServiceError::Unexpected(message) => Status::internal(message),
        }
    }
}

#[tonic::async_trait]
impl ChordNode for ChordService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = chord_proto::PingResponse {};

        Ok(Response::new(reply))
    }

    async fn find_successor(
        &self,
        request: Request<FindSuccessorRequest>,
    ) -> Result<Response<FindSuccessorResponse>, Status> {
        println!("Got a request: {:?}", request.get_ref());

        let result = self
            .node
            .find_successor(request.get_ref().id)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(result.into()))
    }
}

impl From<chord_rs::Node> for FindSuccessorResponse {
    fn from(node: chord_rs::Node) -> Self {
        FindSuccessorResponse {
            id: node.id(),
            node: Some(node.into()),
        }
    }
}

impl From<chord_rs::Node> for chord_proto::Node {
    fn from(node: chord_rs::Node) -> Self {
        chord_proto::Node {
            ip: Some(node.addr().ip().into()),
            port: node.addr().port() as i32,
        }
    }
}

impl From<IpAddr> for chord_proto::IpAddress {
    fn from(ip: IpAddr) -> Self {
        let (version, address) = match ip {
            IpAddr::V4(v4) => (chord_proto::IpVersion::Ipv4, v4.to_string()),
            IpAddr::V6(v6) => (chord_proto::IpVersion::Ipv6, v6.to_string()),
        };

        chord_proto::IpAddress {
            version: version.into(),
            address: address.as_bytes().to_vec(),
        }
    }
}
