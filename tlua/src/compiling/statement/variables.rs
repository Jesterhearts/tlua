use tlua_parser::ast::statement::variables::{
    Attribute,
    LocalVarList,
};

use crate::{
    compiling::{
        compiler::LocalVariableTarget,
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for LocalVarList<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.write_assign_all_locals(
            self.vars.iter().map(|var| match var.attribute {
                Some(Attribute::Const) => LocalVariableTarget::Constant(var.name),
                Some(Attribute::Close) => LocalVariableTarget::Closable(var.name),
                None => LocalVariableTarget::Mutable(var.name),
            }),
            self.initializers.iter(),
        )
    }
}
