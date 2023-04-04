use std::net::SocketAddr;

use clap::{command, Parser, arg, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {

    /// Address of a node in the ring to connect to, format: IP[:PORT], e.g. [::1]:42000
    pub(crate) ring: SocketAddr,

    /// Set the log level
    #[arg(short('L'), long, value_name = "LEVEL", value_enum, default_value_t = LogLevel::Info)]
    pub(crate) log_level: LogLevel,

    /// Sets a log file to write to
    /// If not set, logs will be written to stdout
    #[arg(short, long, value_name = "FILE")]
    pub(crate) log_file: Option<String>,
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
