use tamto_grpc::server::{ChordNodeServer, ChordService, Server};

mod cli;
use cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let addr = cli.listen;
    println!("Listening on: {}", addr);
    let chord = ChordService::new(addr, cli.ring);

    let server = Server::builder()
        .add_service(ChordNodeServer::new(chord))
        .serve(addr);

    server.await?;
    Ok(())
}
