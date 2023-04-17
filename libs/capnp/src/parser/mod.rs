use std::fmt::Display;

mod node;

pub use node::*;

#[derive(Debug)]
pub enum ParserError {
    InvalidNode,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNode => write!(f, "Invalid node"),
        }
    }
}
