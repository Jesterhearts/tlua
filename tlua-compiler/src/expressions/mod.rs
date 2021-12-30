use tlua_bytecode::Constant;
use tlua_parser::ast::{
    constant_string::ConstantString,
    expressions::{
        number::Number,
        Nil,
        VarArgs,
    },
};

use crate::{
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

pub(crate) mod function_defs;
pub(crate) mod operators;
pub(crate) mod tables;

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
        Ok(NodeOutput::Constant(
            tlua_bytecode::Number::from(*self).into(),
        ))
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
