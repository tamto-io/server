use std::net::SocketAddr;

use server::chord_proto;

pub mod client;
pub mod server;


impl TryFrom<chord_proto::Node> for chord_rs::Node {
    type Error = std::net::AddrParseError;

    fn try_from(node: chord_proto::Node) -> Result<Self, Self::Error> {
        let id = node.id;
        let ip = node.ip.unwrap();
        let ip = ip.try_into().unwrap();
        let port = node.port as u16;

        let addr = SocketAddr::new(ip, port);

        Ok(chord_rs::Node::with_id(id, addr))
    }
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
