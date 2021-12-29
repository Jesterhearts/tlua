use tlua_bytecode::OpError;
use tlua_parser::ast::statement::for_loop::ForLoop;

use crate::compiling::{
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for ForLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
