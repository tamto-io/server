use std::net::SocketAddr;

use clap::{arg, command, Args, Parser, Subcommand, ValueEnum};
use tamto_grpc::client::ChordGrpcClient;

use crate::commands::{lookup::Lookup, CommandExecute, CommandResult, Error};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Address of a node in the ring to connect to, format: IP[:PORT], e.g. [::1]:42000
    #[arg(long, value_name = "ADDRESS:PORT")]
    pub(crate) ring: SocketAddr,

    /// Set the log level
    #[arg(short('L'), long, value_name = "LEVEL", value_enum, default_value_t = LogLevel::Warn)]
    pub(crate) log_level: LogLevel,

    /// Subcommand
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Lookup a key in the ring, returns the node that owns the key
    Lookup(LookupArgs),
}

#[async_trait::async_trait]
impl CommandExecute for Commands {
    async fn execute(&self, client: ChordGrpcClient) -> Result<CommandResult, Error> {
        match self {
            Commands::Lookup(args) => {
                let lookup: Lookup = Lookup::try_from(args)?;
                lookup.execute(client).await
            }
        }
    }
}

#[derive(Args)]
pub(crate) struct LookupArgs {
    /// Key to lookup
    pub(crate) key: String,

    /// Whether the key is a raw identifier,
    /// if set, the key MUST be an integer
    #[arg(long, default_value_t = false)]
    pub(crate) raw: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}
