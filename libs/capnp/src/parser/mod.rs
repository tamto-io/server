use std::fmt::Display;

mod node;

pub use node::*;

/// Trait for inserting a value into a Cap'n'proto result builder.
///
/// This is used to map internal types to Cap'n'proto types.
pub(crate) trait ResultBuilder<T>: Sized {
    type Output;

    /// Insert a value into the result builder
    ///
    /// # Arguments
    ///
    /// * `value` - The value to insert.
    ///
    /// # Example
    ///
    /// In this example, we map a `Node` to a `FindSuccessorResults` struct.
    /// ```rust
    /// use chord_rs::Node;
    /// use tamto_capnp::chord_capnp::chord_node::FindSuccessorResults;
    ///
    /// impl ResultBuilder<Node> for chord_capnp::chord_node::FindSuccessorResults {
    ///     fn insert(mut self, value: Node) -> Result<Self::Output, capnp::Error> {
    ///         let mut node = self.get().init_node();
    ///         node.set_id(value.id().into());
    ///
    ///         Ok(())
    ///    }
    /// }
    /// ```
    fn insert(self, value: T) -> Result<Self::Output, capnp::Error>;
}

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
