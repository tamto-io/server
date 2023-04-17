use chord_rs::Client;
use clap::Parser;
use commands::{CommandResult, Error};
use tamto_grpc::client::ChordGrpcClient;
use tamto_capnp::client::ChordCapnpClient;

use crate::{cli::Cli, commands::CommandExecute};

mod cli;
mod commands;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(result) => print_result(result),
        Err(err) => println!("Failed:\n {}", err),
    }
}

async fn run(cli: Cli) -> Result<CommandResult, Error> {
    // let client = ChordGrpcClient::init(cli.ring).await;
    let client = ChordCapnpClient::init(cli.ring).await;

    CommandExecute::execute(&cli.command, client).await
}

fn print_result(result: CommandResult) {
    println!("{}", result.result);
    println!("Execution time: {:?}", result.execution);
}
