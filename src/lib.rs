use nom_supreme::error::ErrorTree;
use thiserror::Error;
use tracing::instrument;

pub mod ast;
pub mod compiling;
pub mod list;
pub mod parsing;
pub mod values;
pub mod vm;

use self::{
    ast::ASTAllocator,
    compiling::{
        compiler::Compiler,
        Chunk,
    },
    parsing::LuaParseError,
};

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LuaError {
    #[error("syntax error: {0}")]
    SyntaxError(String),
    #[error("vm execution error")]
    ExecutionError(vm::OpError),
}

impl From<vm::OpError> for LuaError {
    fn from(err: vm::OpError) -> Self {
        Self::ExecutionError(err)
    }
}

impl From<ErrorTree<LuaParseError>> for LuaError {
    fn from(err: ErrorTree<LuaParseError>) -> Self {
        Self::SyntaxError(err.to_string())
    }
}

#[instrument(level = "trace", name="compile", skip(src), fields(src_bytes = src.as_bytes().len()))]
pub fn compile(src: &str) -> Result<Chunk, LuaError> {
    let alloc = ASTAllocator::default();

    let ast = parsing::parse_chunk(src, &alloc)?;

    let compiler = Compiler::default();

    Ok(compiler.compile_ast(ast).expect("Internal compiler error"))
}
