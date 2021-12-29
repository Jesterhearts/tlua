use thiserror::Error;
pub use tlua_bytecode::OpError;
use tlua_parser::{
    ast::ASTAllocator,
    parsing::{
        parse_chunk,
        ChunkParseError,
    },
};
use tracing::instrument;

pub mod compiling;
pub mod values;
pub mod vm;

use self::compiling::{
    compiler::Compiler,
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

#[instrument(level = "trace", name="compile", skip(src), fields(src_bytes = src.as_bytes().len()))]
pub fn compile(src: &str) -> Result<Chunk, LuaError> {
    let alloc = ASTAllocator::default();

    let ast = parse_chunk(src, &alloc)?;

    let compiler = Compiler::default();

    Ok(compiler.compile_ast(ast).expect("Internal compiler error"))
}
