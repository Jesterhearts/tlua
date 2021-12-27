use crate::{
    ast::{
        prefix_expression::VarPrefixExpression,
        statement::assignment::Assignment,
    },
    compiling::{
        compiler::VariableTarget,
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

impl CompileStatement for Assignment<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.write_assign_all(
            self.varlist.iter().map(|var| match var {
                VarPrefixExpression::Name(name) => VariableTarget::Ident(*name),
                VarPrefixExpression::TableAccess { .. } => todo!(),
            }),
            self.expressions.iter(),
        )
    }
}
