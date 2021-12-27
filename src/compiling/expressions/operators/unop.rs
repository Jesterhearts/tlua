use crate::{
    ast::expressions::operator::*,
    compiling::{
        CompileError,
        CompileExpression,
        CompilerContext,
        NodeOutput,
    },
};

impl CompileExpression for Negation<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}

impl CompileExpression for Not<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}

impl CompileExpression for BitNot<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}

impl CompileExpression for Length<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}
