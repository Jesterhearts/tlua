use tlua_bytecode::OpError;
use tlua_parser::ast::statement::variables::LocalVarList;

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
            |compiler, var| match var.attribute {
                None => compiler.new_local(var.name),
                Some(_) => todo!(),
            },
            self.vars.iter(),
            self.initializers.iter(),
        )
    }
}
