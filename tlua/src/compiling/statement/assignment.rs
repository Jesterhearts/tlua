use tlua_bytecode::OpError;
use tlua_parser::ast::{
    prefix_expression::VarPrefixExpression,
    statement::assignment::Assignment,
};

use crate::compiling::{
    compiler::VariableTarget,
    CompileError,
    CompileStatement,
    CompilerContext,
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
