use std::{fmt::Display, time::Duration};

use chord_rs::{client::ClientError, Client};

pub(crate) mod lookup;
pub(crate) mod ping;

#[async_trait::async_trait]
pub trait CommandExecute {
    async fn execute<C>(&self, client: C) -> Result<CommandResult, Error>
    where
        C: Client + Clone + Send + Sync;
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

pub struct CommandResult {
    pub(crate) result: String,
    pub(crate) execution: Duration,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Self {
        Self {
            message: format!("Client error: {err:?}"),
        }
    }
}
