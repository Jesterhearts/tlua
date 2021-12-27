use crate::{
    ast::statement::while_loop::WhileLoop,
    compiling::{
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for WhileLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
