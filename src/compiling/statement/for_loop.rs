use crate::{
    ast::statement::for_loop::ForLoop,
    compiling::{
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for ForLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
