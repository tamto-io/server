use tamto_grpc::server::{ChordNodeServer, ChordService, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse()?;
    let chord = ChordService::new(addr);

    let server = Server::builder()
        .add_service(ChordNodeServer::new(chord))
        .serve(addr);

    server.await?;
    Ok(())
}
