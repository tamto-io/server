use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
// use tamto_capnp::Server as CapnpServer;
use chord_rs::Server;

mod cli;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let cli = Cli::parse();

    let addr = cli.listen;
    println!("Listening on: {}", addr);
    // let chord = ChordService::new(addr, cli.ring).await;

    // let server = Server::builder()
    //     .add_service(ChordNodeServer::new(chord))
    //     .serve(addr);

    // server.await?;

    let server = Server::new(addr, cli).await;

    server.run().await;
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
