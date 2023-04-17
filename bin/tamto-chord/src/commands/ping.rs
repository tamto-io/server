use std::num::ParseIntError;

use chord_rs::{Client};

use crate::cli::PingArgs;

use super::{CommandExecute, CommandResult, Error};

pub(crate) struct Ping {
}

#[async_trait::async_trait]
impl CommandExecute for Ping {
    async fn execute<C>(&self, client: C) -> Result<CommandResult, Error> where C: Client + Clone + Send + Sync {
        let start = std::time::Instant::now();
        client.ping().await?;

        let elapsed = start.elapsed();
        let result = CommandResult {
            result: format!(
                "Pong",
            ),
            execution: elapsed,
        };

        Ok(result)
    }
}

impl TryFrom<&PingArgs> for Ping {
    type Error = PingError;

    fn try_from(_: &PingArgs) -> Result<Self, Self::Error> {
        Ok(Ping {})
    }
}

impl From<ParseIntError> for PingError {
    fn from(error: ParseIntError) -> Self {
        PingError::KeyParseError(error.to_string())
    }
}

impl From<PingError> for Error {
    fn from(err: PingError) -> Self {
        match err {
            PingError::KeyParseError(msg) => Error {
                message: format!("Failed to parse key: {}", msg),
            },
        }
    }
}

pub enum PingError {
    KeyParseError(String),
}
