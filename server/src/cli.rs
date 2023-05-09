use std::net::SocketAddr;

use chord_rs::Config;
use clap::{arg, command, Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Sets a socket address to listen on
    #[arg(short, long, value_name = "[ADDRESS[:PORT]]", default_value_t = SocketAddr::from(([127, 0, 0, 1], 42000)))]
    pub(crate) listen: SocketAddr,

    /// Address of a node in the ring to join
    #[arg(short, long, value_name = "[ADDRESS[:PORT]]")]
    pub(crate) ring: Option<SocketAddr>,

    /// Set the log level
    #[arg(short('L'), long, value_name = "LEVEL", value_enum, default_value_t = LogLevel::Info)]
    pub(crate) log_level: LogLevel,

    /// Set the maximum number of concurrent connections
    /// (default: 1024)
    #[arg(long, value_name = "CONNECTIONS", default_value = "1024")]
    pub(crate) max_connections: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Into<Config> for Cli {
    fn into(self) -> Config {
        Config {
            addr: self.listen,
            ring: self.ring,
            max_connections: self.max_connections,
        }
    }
}
