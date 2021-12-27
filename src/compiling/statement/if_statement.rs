use crate::{
    ast::statement::if_statement::If,
    compiling::{
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for If<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.write_if_sequence(
            std::iter::once(&self.cond).chain(self.elif.iter().map(|elif| &elif.cond)),
            std::iter::once(&self.body)
                .chain(self.elif.iter().map(|elif| &elif.body))
                .chain(self.else_final.as_ref()),
        )
    }
}
