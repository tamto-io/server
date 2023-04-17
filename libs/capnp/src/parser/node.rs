use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};

use chord_rs::Node;

use crate::chord_capnp::chord_node::node::ip_address;

use super::super::chord_capnp::chord_node::node;

impl TryFrom<node::Reader<'_>> for Node {
    type Error = super::ParserError;

    fn try_from(value: node::Reader<'_>) -> Result<Self, Self::Error> {
        let id = value.get_id();
        let addr: SocketAddr = value.get_address().unwrap().into();
        
        Ok(Node::with_id(id.into(), addr))
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
