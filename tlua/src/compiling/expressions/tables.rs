use tlua_parser::ast::expressions::tables::TableConstructor;

use crate::compiling::{
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
