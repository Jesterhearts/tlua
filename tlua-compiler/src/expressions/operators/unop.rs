use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::{
        self,
        UnaryBitNot,
        UnaryMinus,
    },
    Truthy,
};
use tlua_parser::ast::expressions::operator::*;

use crate::{
    constant::Constant,
    CompileError,
    CompileExpression,
    NodeOutput,
    Scope,
};

impl CompileExpression for Negation<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        scope.write_unary_op::<UnaryMinus, _, _>(&self.0, |v| match v {
            Constant::Float(f) => Ok((-f).into()),
            Constant::Integer(i) => Ok((-i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "negation" }),
        })
    }
}

impl CompileExpression for BitNot<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        scope.write_unary_op::<UnaryBitNot, _, _>(&self.0, |v| match v {
            Constant::Float(f) => f64inbounds(f).map(|i| (!i).into()),
            Constant::Integer(i) => Ok((!i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "bitwise not" }),
        })
    }
}

impl CompileExpression for Not<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        scope.write_unary_op::<opcodes::Not, _, _>(&self.0, |v| Ok((!v.as_bool()).into()))
    }
}

impl CompileExpression for Length<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        scope.write_unary_op::<opcodes::Length, _, _>(&self.0, |v| match v {
            Constant::String(s) => i64::try_from(s.len())
                .map(Constant::from)
                .map_err(|_| tlua_bytecode::OpError::StringLengthOutOfBounds),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "length" }),
        })
    }
}
