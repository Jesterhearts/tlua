use tlua_parser::ast::{
    constant_string::ConstantString,
    expressions::{
        number::Number,
        Nil,
        VarArgs,
    },
};

use crate::{
    compiling::{
        CompileExpression,
        CompilerContext,
        NodeOutput,
    },
    vm::{
        self,
        Constant,
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
        Ok(NodeOutput::Constant(
            match *self {
                Number::Float(f) => vm::Number::from(f),
                Number::Integer(i) => vm::Number::from(i),
            }
            .into(),
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
