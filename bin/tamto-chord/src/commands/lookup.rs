use std::num::ParseIntError;

use chord_rs::NodeId;
use tamto_grpc::client::ChordGrpcClient;

use crate::cli::LookupArgs;

use super::{CommandExecute, Error, CommandResult};

pub(crate) struct Lookup {
    key: NodeId
}

#[async_trait::async_trait]
impl CommandExecute for Lookup {
    async fn execute(&self, client: ChordGrpcClient) -> Result<CommandResult, Error> {
        let start = std::time::Instant::now();
        let node = client.find_successor(self.key.into()).await?;

        let elapsed = start.elapsed();
        let result = CommandResult {
            result: format!("Key: {}\nNode:\n  Address: {}\n  Id: {}", self.key, node.addr(), node.id()),
            execution: elapsed,
        };

        Ok(result)
    }
}

impl TryFrom<&LookupArgs> for Lookup {
    type Error = LookupError;

    fn try_from(args: &LookupArgs) -> Result<Self, Self::Error> {
        let key = if args.raw {
            NodeId::from(args.key.parse::<u64>()?)
        } else {
            NodeId::from(args.key.clone())
        };

        Ok(Lookup { key })
    }
}

impl From<ParseIntError> for LookupError {
    fn from(error: ParseIntError) -> Self {
        LookupError::KeyParseError(error.to_string())
    }
}

impl From<LookupError> for Error {
    fn from(err: LookupError) -> Self {
        match err {
            LookupError::KeyParseError(msg) => Error {
                message: format!("Failed to parse key: {}", msg),
            },
        }
    }
}

pub enum LookupError {
    KeyParseError(String),
}
