use std::fmt::Display;

mod node;
mod errors;
pub use node::*;

/// Trait for inserting a value into a Cap'n'proto result builder.
///
/// This is used to map internal types to Cap'n'proto types.
pub trait ResultBuilder<T>: Sized {
    type Output;

    /// Insert a value into the result builder
    ///
    /// # Arguments
    ///
    /// * `value` - The value to insert.
    fn insert(self, value: T) -> Result<Self::Output, capnp::Error>;
}

#[derive(Debug)]
pub enum ParserError {
    InvalidNode,
    InvalidIp(String),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNode => write!(f, "Invalid node"),
            Self::InvalidIp(msg) => write!(f, "{}", msg),
        }
    }
}
