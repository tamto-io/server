use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use capnp::message;
use chord_rs::Node;

use crate::chord_capnp;
use crate::chord_capnp::chord_node::node::ip_address;

use crate::chord_capnp::chord_node::node;

use super::ResultBuilder;

/// Map a capnp node to a chord_rs node
impl TryFrom<node::Reader<'_>> for Node {
    type Error = super::ParserError;

    fn try_from(value: node::Reader<'_>) -> Result<Self, Self::Error> {
        let id = value.get_id();
        let addr: SocketAddr = value.get_address().unwrap().into();

        Ok(Node::with_id(id.into(), addr))
    }
}

/// Map capnp ip_address to a std::net::SocketAddr
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

/// Insert a `Node` into a `FindSuccessorResults` struct.
impl ResultBuilder<Node> for chord_capnp::chord_node::FindSuccessorResults {
    type Output = ();
    #[inline]
    fn insert(mut self, value: Node) -> Result<Self::Output, capnp::Error> {
        let mut node = self.get().init_node();
        node.set_id(value.id().into());

        node.init_address().insert(value.addr())?;

        Ok(())
    }
}

/// Insert a `SocketAddr` into a `IpAddress` struct.
impl ResultBuilder<SocketAddr> for chord_capnp::chord_node::node::ip_address::Builder<'_> {
    type Output = ();

    #[inline]
    fn insert(mut self, value: SocketAddr) -> Result<Self::Output, capnp::Error> {
        self.set_port(value.port());
        self.insert(value.ip())?;

        Ok(())
    }
}

impl<'a> ResultBuilder<IpAddr> for chord_capnp::chord_node::node::ip_address::Builder<'a> {
    type Output = ();

    #[inline]
    fn insert(self, value: IpAddr) -> Result<Self::Output, capnp::Error> {
        match value {
            IpAddr::V4(v4) => {
                //builder.insert(v4)?;
                let octets: Vec<u8> = v4.octets().to_vec();
                let mut ip = self.init_ipv4(4);
                for i in 0..4 {
                    ip.set(i, octets[i as usize]);
                }
            }
            IpAddr::V6(v6) => {
                let segments: Vec<u16> = v6.segments().to_vec();
                let mut ip = self.init_ipv6(8);
                for i in 0..8 {
                    ip.set(i, segments[i as usize]);
                }
            }
        }

        Ok(())
    }
}

mod tests {

    use super::*;

    #[test]
    fn test_socket_addr_ipv4_to_ip_address() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let mut message = message::Builder::new_default();
        let builder = message.init_root::<chord_capnp::chord_node::node::ip_address::Builder<'_>>();
        builder.insert(addr).unwrap();

        let reader: chord_capnp::chord_node::node::ip_address::Reader =
            message.get_root_as_reader().unwrap();
        assert_eq!(reader.get_port(), 8080);
        assert_eq!(reader.has_ipv6(), false);
        assert_eq!(reader.has_ipv4(), true);
    }

    #[test]
    fn test_socket_addr_ipv6_to_ip_address() {
        let addr: SocketAddr = "[::1]:8080".parse().unwrap();
        let mut message = message::Builder::new_default();
        let builder = message.init_root::<chord_capnp::chord_node::node::ip_address::Builder<'_>>();
        builder.insert(addr).unwrap();

        let reader: chord_capnp::chord_node::node::ip_address::Reader =
            message.get_root_as_reader().unwrap();
        assert_eq!(reader.get_port(), 8080);
        assert_eq!(reader.has_ipv6(), true);
        assert_eq!(reader.has_ipv4(), false);
    }
}
