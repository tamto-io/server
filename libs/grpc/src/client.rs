use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};

use crate::server::chord_proto::chord_node_client::ChordNodeClient;
use crate::server::chord_proto::{FindSuccessorRequest, self};
use chord_rs::client::ClientError;
use chord_rs::{Client, Node};
use tonic::async_trait;
use tonic::transport::Endpoint;

#[derive(Debug)]
pub struct ChordGrpcClient {
    endpoint: Endpoint,
    // client: ChordNodeClient<Channel>,
}

#[async_trait]
impl Client for ChordGrpcClient {
    fn init(addr: SocketAddr) -> Self {
        // TODO: the protocol should be configurable
        let endpoint = Endpoint::from_shared(format!("http://{}", addr)).unwrap();

        ChordGrpcClient { endpoint }
    }

    async fn find_successor(&self, id: u64) -> Result<Node, ClientError> {
        let mut client = ChordNodeClient::connect(self.endpoint.clone())
            .await
            .unwrap();

        // let mut client = ChordNodeClient::new(self.channel.clone());

        let request = tonic::Request::new(FindSuccessorRequest { id });
        let response = client.find_successor(request).await.unwrap().into_inner();

        let node = response.node.unwrap();
        let node: Node = node.try_into().unwrap();

        println!("response: {:?}", node.addr());

        Ok(node)
    }

    fn successor(&self) -> Result<Node, ClientError> {
        unimplemented!()
    }

    fn predecessor(&self) -> Result<Option<Node>, ClientError> {
        unimplemented!()
    }

    fn notify(&self, _predecessor: Node) -> Result<(), ClientError> {
        unimplemented!()
    }

    fn ping(&self) -> Result<(), ClientError> {
        unimplemented!()
    }
}

impl TryFrom<chord_proto::Node> for chord_rs::Node {
    type Error = std::net::AddrParseError;

    fn try_from(node: chord_proto::Node) -> Result<Self, Self::Error> {
        let ip = node.ip.unwrap();
        let ip = ip.try_into().unwrap();
        let port = node.port as u16;

        let addr = SocketAddr::new(ip, port);

        Ok(chord_rs::Node::new(addr))
    }
}

impl ChordGrpcClient {
    pub fn new(addr: SocketAddr) -> Self {
        Self::init(addr)
    }

    pub async fn find_successor(&self, id: u64) -> Result<Node, ClientError> {
        Client::find_successor(self, id).await
    }
}

#[derive(Debug)]
pub struct IpParseError{
    msg: String,
}

impl IpParseError {
    fn new(msg: &str) -> Self {
        IpParseError { msg: msg.to_string() }
    }
}

impl std::fmt::Display for IpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl TryFrom<chord_proto::IpAddress> for IpAddr {
    type Error = IpParseError;

    fn try_from(ip: chord_proto::IpAddress) -> Result<Self, Self::Error> {
        
        fn ipv4(addr: Vec<u8>) -> [u8; 4] {
            let mut array = [0; 4];
            array.copy_from_slice(&addr);
            return array
        }

        fn ipv6(addr: Vec<u8>) -> [u8; 16] {
            let mut array = [0; 16];
            array.copy_from_slice(&addr);
            return array
        }

        if ip.is_v4() && ip.address.len() != 4 {
            return Err(IpParseError::new("Invalid IPv4 address"));
        } else if ip.is_v6() && ip.address.len() != 16 {
            return Err(IpParseError::new("Invalid IPv6 address"));
        } else if ip.is_v4() {
            return Ok(IpAddr::V4(Ipv4Addr::from(ipv4(ip.address))));
        } else if ip.is_v6() {
            return Ok(IpAddr::V6(Ipv6Addr::from(ipv6(ip.address))));
        } else {
            return Err(IpParseError::new("Invalid IP address"));
        }
    }
}

impl chord_proto::IpAddress {
    fn is_v4(&self) -> bool {
        self.version == chord_proto::IpVersion::Ipv4 as i32
    }

    fn is_v6(&self) -> bool {
        self.version == chord_proto::IpVersion::Ipv6 as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ipv4() {
        fn addr(addr: Vec<u8>) -> chord_proto::IpAddress {
            chord_proto::IpAddress {
                version: chord_proto::IpVersion::Ipv4 as i32,
                address: addr,
            }
        }

        let valid_ip = addr(vec![127, 0, 0, 1]);
        let invalid_ip = IpAddr::try_from(addr(vec![127, 0, 0, 1, 2]));

        assert_eq!(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), IpAddr::try_from(valid_ip).unwrap());
        assert!(invalid_ip.is_err());
        assert_eq!("Invalid IPv4 address", invalid_ip.err().unwrap().msg);

        let invalid_ip = IpAddr::try_from(addr(vec![127, 0]));
        assert!(invalid_ip.is_err());
        assert_eq!("Invalid IPv4 address", invalid_ip.err().unwrap().msg);
    }

    #[test]
    fn parse_ipv6() {
        fn addr(addr: Vec<u8>) -> chord_proto::IpAddress {
            chord_proto::IpAddress {
                version: chord_proto::IpVersion::Ipv6 as i32,
                address: addr,
            }
        }

        let ipv6: Ipv6Addr = "fd9f:9b7:9d0e::".parse().unwrap();

        let mut valid_bytes = vec![253, 159, 9, 183, 157, 14];
        valid_bytes.resize(16, 0);
        let valid_ip = addr(valid_bytes);
        let invalid_ip = IpAddr::try_from(addr(vec![127, 0, 0, 1, 2]));

        assert_eq!(ipv6, IpAddr::try_from(valid_ip).unwrap());
        assert!(invalid_ip.is_err());
        assert_eq!("Invalid IPv6 address", invalid_ip.err().unwrap().msg);

        let invalid_ip = IpAddr::try_from(addr(vec![127, 0]));
        assert!(invalid_ip.is_err());
        assert_eq!("Invalid IPv6 address", invalid_ip.err().unwrap().msg);
    }
}
