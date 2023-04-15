use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use tamto_grpc::server::{ChordNodeServer, ChordService, Server};

mod cli;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let cli = Cli::parse();

    let addr = cli.listen;
    println!("Listening on: {}", addr);
    let chord = ChordService::new(addr, cli.ring).await;

    let server = Server::builder()
        .add_service(ChordNodeServer::new(chord))
        .serve(addr);

    server.await?;
    Ok(())
}

fn setup_logging() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    log::info!("Logging started");
}
