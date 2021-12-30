use tlua_bytecode::OpError;
use tlua_parser::ast::statement::foreach_loop::ForEachLoop;

use crate::{
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for ForEachLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
