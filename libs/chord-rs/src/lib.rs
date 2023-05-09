
#[cfg(all(feature = "capnp", feature = "grpc"))]
compile_error!("feature \"capnp\" and feature \"grpc\" cannot be enabled at the same time");

use std::net::SocketAddr;

#[cfg(feature = "grpc")]
pub use grpc::Server;

#[cfg(feature = "capnp")]
pub use capnp::Server;

pub struct Config {
    pub addr: SocketAddr,
    pub ring: Option<SocketAddr>,

    pub max_connections: usize,
}

#[cfg(feature = "capnp")]
mod capnp {
    use std::net::SocketAddr;

    use crate::Config;
    use tamto_capnp::Server as CapnpServer;

    pub struct Server {
        server: CapnpServer,
        config: Config,
    }

    impl Server {
        pub async fn new(addr: SocketAddr, config: impl Into<Config>) -> Server {
            let config: Config = config.into();
            let chord = CapnpServer::new(addr, config.ring).await;

            Server {
                server: chord,
                config
            }
        }

        pub async fn run(self) {
            self.server.run(self.config.max_connections).await;
        }
    }
}

#[cfg(feature = "grpc")]
mod grpc {
    use std::net::SocketAddr;
    use tamto_grpc::server::ChordNodeServer;
    use tamto_grpc::server::Server as GrpcServer;
    use tamto_grpc::server::ChordService;

    use crate::Config;

    pub struct Server {
        addr: SocketAddr,
        router: tonic::transport::server::Router,
    }

    impl Server {
        pub async fn new(addr: SocketAddr, config: impl Into<Config>) -> Server {
            let config: Config = config.into();
            let chord = ChordService::new(addr, config.ring).await;
    
            let router = GrpcServer::builder()
                .add_service(ChordNodeServer::new(chord));
    
            Server {
                addr,
                router
            }
        }
    
        pub async fn run(self) {
            match self.router.serve(self.addr).await {
                Ok(_) => log::info!("Server stopped"),
                Err(e) => log::error!("Server error: {}", e),
            }
        }    
    }
}
