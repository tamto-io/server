use chord_rs::client::ClientError;

use crate::client::CapnpClientError;

use super::ParserError;

impl From<ParserError> for CapnpClientError {
    fn from(value: ParserError) -> Self {
        CapnpClientError::InvalidRequest(value.to_string())
    }
}

impl From<capnp::Error> for CapnpClientError {
    fn from(value: capnp::Error) -> Self {
        log::error!("capnp error: {:?}", value);
        CapnpClientError::Unexpected(value.to_string())
    }
}

impl From<capnp::NotInSchema> for CapnpClientError {
    fn from(value: capnp::NotInSchema) -> Self {
        log::error!("value not in schema: {}", value);
        CapnpClientError::Unexpected(value.to_string())
    }
}

impl From<CapnpClientError> for ClientError {
    fn from(value: CapnpClientError) -> Self {
        match value {
            CapnpClientError::Unexpected(message) => Self::Unexpected(message),
            CapnpClientError::InvalidRequest(message) => Self::InvalidRequest(message),
            CapnpClientError::ConnectionFailed(_) => todo!(),
        }
    }
}
