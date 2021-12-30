use thiserror::Error;
pub use tlua_bytecode::OpError;
use tlua_parser::parsing::ChunkParseError;

pub mod values;
pub mod vm;

pub use tlua_compiler::{
    compile,
    Chunk,
};

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LuaError {
    #[error("syntax error: {0}")]
    SyntaxError(String),
    #[error("vm execution error")]
    ExecutionError(OpError),
}

impl From<OpError> for LuaError {
    fn from(err: OpError) -> Self {
        Self::ExecutionError(err)
    }
}

impl From<ChunkParseError> for LuaError {
    fn from(err: ChunkParseError) -> Self {
        Self::SyntaxError(err.to_string())
    }
}
