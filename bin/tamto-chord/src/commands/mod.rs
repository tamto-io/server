use std::{fmt::Display, time::Duration};

use chord_rs::client::ClientError;
use tamto_grpc::client::ChordGrpcClient;

pub(crate) mod lookup;

#[async_trait::async_trait]
pub trait CommandExecute {
    async fn execute(&self, client: ChordGrpcClient) -> Result<CommandResult, Error>;
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
            message: format!("Client error: {}", err),
        }
    }
}
