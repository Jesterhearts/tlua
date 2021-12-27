use crate::{
    ast::statement::repeat_loop::RepeatLoop,
    compiling::{
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for RepeatLoop<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
