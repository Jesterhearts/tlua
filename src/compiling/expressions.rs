use crate::{
    ast::{
        constant_string::ConstantString,
        expressions::VarArgs,
    },
    compiling::{
        CompileExpression,
        CompilerContext,
        NodeOutput,
    },
    values::Nil,
    vm::{
        Constant,
        Number,
    },
};

pub mod function_defs;
pub mod operators;
pub mod tables;

impl CompileExpression for Nil {
    fn compile(&self, _: &mut CompilerContext) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(Constant::Nil))
    }
}

impl CompileExpression for bool {
    fn compile(&self, _: &mut CompilerContext) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(Constant::Bool(*self)))
    }
}

impl CompileExpression for Number {
    fn compile(&self, _: &mut CompilerContext) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant((*self).into()))
    }
}

impl CompileExpression for ConstantString {
    fn compile(&self, _: &mut CompilerContext) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(Constant::String(*self)))
    }
}

impl CompileExpression for VarArgs {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, super::CompileError> {
        compiler.check_varargs()?;
        Ok(NodeOutput::VAStack)
    }
}
