use crate::node::store::NodeStore;
use crate::Client;
use std::marker::PhantomData;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct NodeService<C: Client> {
    id: u64,
    addr: SocketAddr,
    store: NodeStore,
    phantom: PhantomData<C>,
}

#[cfg(not(feature = "async"))]
pub mod sync;

pub mod error {
    use crate::client;
    use std::fmt::Display;

    #[derive(Debug)]
    pub enum ServiceError {
        Unexpected(String),
    }

    impl From<client::ClientError> for ServiceError {
        fn from(err: client::ClientError) -> Self {
            Self::Unexpected(format!("Client error: {}", err))
        }
    }

    impl Display for ServiceError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Unexpected(message) => write!(f, "{}", message),
            }
        }
    }
}
