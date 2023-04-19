use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

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
        let addr: SocketAddr = value.get_address().unwrap().try_into()?;

        Ok(Node::with_id(id.into(), addr))
    }
}

/// Map capnp ip_address to a std::net::SocketAddr
impl TryFrom<ip_address::Reader<'_>> for SocketAddr {
    type Error = super::ParserError;

    fn try_from(addr: ip_address::Reader<'_>) -> Result<Self, Self::Error> {
        let port = addr.get_port();
        let address = match addr.which().unwrap() {
            ip_address::Which::Ipv4(Ok(ipv4)) => {
                let mut array = [0; 4];
                if let Some(ip) = ipv4.as_slice() {
                    if ip.len() != 4 {
                        return Err(super::ParserError::InvalidIp("IPv4 should contain 4 chunks".to_string()));
                    }
                    array.copy_from_slice(ip);
                    Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::from(array)), port))
                } else {
                    Err(super::ParserError::InvalidIp("Error parsing ipv4 address".to_string()))
                }
            }
            ip_address::Which::Ipv6(Ok(ipv6)) => {
                let mut array = [0; 8];
                if let Some(ip) = ipv6.as_slice() {
                    if ip.len() != 8 {
                        return Err(super::ParserError::InvalidIp("IPv6 should contain 8 chunks, each containing u16".to_string()));
                    }
                    array.copy_from_slice(ip);
                    Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::from(array)), port))
                } else {
                    Err(super::ParserError::InvalidIp("Error parsing IPv6 address".to_string()))
                }
            }
            ip_address::Which::Ipv4(Err(err)) => {
                Err(super::ParserError::InvalidIp(format!("Error parsing ipv4 address: {}", err)))
            }
            ip_address::Which::Ipv6(Err(err)) => {
                Err(super::ParserError::InvalidIp(format!("Error parsing ipv6 address: {}", err)))
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
    use std::net::SocketAddr;
    use capnp::message;
    use crate::{chord_capnp, parser::ResultBuilder};



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

        let ip = SocketAddr::try_from(reader).unwrap();

        assert_eq!(ip, addr);
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

        let ip = SocketAddr::try_from(reader).unwrap();

        assert_eq!(ip, addr);
    }

    #[test]
    fn test_invalid_ip_to_deserialization() {
        let message = message::Builder::new_default();

        let reader: chord_capnp::chord_node::node::ip_address::Reader =
            message.get_root_as_reader().unwrap();

        let ip = SocketAddr::try_from(reader);

        assert!(ip.is_err());
        assert_eq!(ip.unwrap_err().to_string(), "Error parsing ipv4 address".to_string());
    }

    #[test]
    fn test_invalid_ipv6_to_deserialization() {
        let mut message = message::Builder::new_default();
        let mut builder = message.init_root::<chord_capnp::chord_node::node::ip_address::Builder<'_>>();
        builder.set_port(8080);
        let mut ip_builder = builder.init_ipv6(4);
        ip_builder.set(0, 0);

        let reader: chord_capnp::chord_node::node::ip_address::Reader =
            message.get_root_as_reader().unwrap();

        let ip = SocketAddr::try_from(reader);

        assert!(ip.is_err());
        assert_eq!(ip.unwrap_err().to_string(), "IPv6 should contain 8 chunks, each containing u16".to_string());
    }

    #[test]
    fn test_invalid_ipv4_to_deserialization() {
        let mut message = message::Builder::new_default();
        let mut builder = message.init_root::<chord_capnp::chord_node::node::ip_address::Builder<'_>>();
        builder.set_port(8080);
        let mut ip_builder = builder.init_ipv4(2);
        ip_builder.set(0, 0);

        let reader: chord_capnp::chord_node::node::ip_address::Reader =
            message.get_root_as_reader().unwrap();

        let ip = SocketAddr::try_from(reader);

        assert!(ip.is_err());
        assert_eq!(ip.unwrap_err().to_string(), "IPv4 should contain 4 chunks".to_string());
    }
}
