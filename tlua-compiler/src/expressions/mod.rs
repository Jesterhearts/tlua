use tlua_bytecode::Constant;
use tlua_parser::expressions::{
    number::Number,
    strings::ConstantString,
    Nil,
    VarArgs,
};

use crate::{
    CompileExpression,
    NodeOutput,
    Scope,
};

pub(crate) mod function_defs;
pub(crate) mod operators;
pub(crate) mod tables;

impl CompileExpression for Nil {
    fn compile(&self, _: &mut Scope) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(Constant::Nil))
    }
}

impl CompileExpression for bool {
    fn compile(&self, _: &mut Scope) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(Constant::Bool(*self)))
    }
}

impl CompileExpression for Number {
    fn compile(&self, _: &mut Scope) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(
            tlua_bytecode::Number::from(*self).into(),
        ))
    }
}

impl CompileExpression for ConstantString {
    fn compile(&self, _: &mut Scope) -> Result<NodeOutput, super::CompileError> {
        Ok(NodeOutput::Constant(Constant::String(*self)))
    }
}

impl CompileExpression for VarArgs {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, super::CompileError> {
        scope.check_varargs()?;
        Ok(NodeOutput::VAStack)
    }
}
