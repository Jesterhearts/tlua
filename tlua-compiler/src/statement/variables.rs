use either::Either;
use tlua_bytecode::OpError;
use tlua_parser::ast::statement::variables::{
    LocalVar,
    LocalVarList,
};

use crate::{
    statement::assignment,
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for LocalVarList<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        assignment::emit_assignments(
            compiler,
            &mut |compiler, var: &LocalVar| match var.attribute {
                None => compiler.new_local(var.name).map(Either::Left),
                Some(_) => todo!(),
            },
            self.vars.iter(),
            self.initializers.iter(),
        )
    }
}
