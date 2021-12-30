use tlua_parser::ast::expressions::tables::TableConstructor;

use crate::{
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, _: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}
