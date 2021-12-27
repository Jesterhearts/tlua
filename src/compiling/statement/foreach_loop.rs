use crate::{
    ast::statement::foreach_loop::ForEachLoop,
    compiling::{
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for ForEachLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
