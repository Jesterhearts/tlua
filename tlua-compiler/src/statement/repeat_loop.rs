use tlua_bytecode::OpError;
use tlua_parser::ast::statement::repeat_loop::RepeatLoop;

use crate::{
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for RepeatLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
