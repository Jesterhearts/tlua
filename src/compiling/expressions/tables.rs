use crate::{
    ast::expressions::tables::TableConstructor,
    compiling::{
        CompileError,
        CompileExpression,
        CompilerContext,
        NodeOutput,
    },
};

impl CompileExpression for TableConstructor<'_> {
    fn compile(&self, _: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}
