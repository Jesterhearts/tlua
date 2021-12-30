use tlua_bytecode::OpError;
use tlua_parser::ast::statement::while_loop::WhileLoop;

use crate::{
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for WhileLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
