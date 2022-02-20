use tlua_parser::expressions::operator::{
    BinaryOperator,
    UnaryOperator,
};

use crate::{
    compiler::Scope,
    CompileError,
    CompileExpression,
    NodeOutput,
};

pub(crate) mod binop;
pub(crate) mod unop;

impl CompileExpression for UnaryOperator<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        match self {
            UnaryOperator::Minus(expr) => expr.compile(scope),
            UnaryOperator::Not(expr) => expr.compile(scope),
            UnaryOperator::Length(expr) => expr.compile(scope),
            UnaryOperator::BitNot(expr) => expr.compile(scope),
        }
    }
}

impl CompileExpression for BinaryOperator<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        match self {
            BinaryOperator::Plus(expr) => expr.compile(scope),
            BinaryOperator::Minus(expr) => expr.compile(scope),
            BinaryOperator::Times(expr) => expr.compile(scope),
            BinaryOperator::Divide(expr) => expr.compile(scope),
            BinaryOperator::IDiv(expr) => expr.compile(scope),
            BinaryOperator::Modulo(expr) => expr.compile(scope),
            BinaryOperator::Exponetiation(expr) => expr.compile(scope),
            BinaryOperator::BitAnd(expr) => expr.compile(scope),
            BinaryOperator::BitOr(expr) => expr.compile(scope),
            BinaryOperator::BitXor(expr) => expr.compile(scope),
            BinaryOperator::ShiftLeft(expr) => expr.compile(scope),
            BinaryOperator::ShiftRight(expr) => expr.compile(scope),
            BinaryOperator::Concat(expr) => expr.compile(scope),
            BinaryOperator::LessThan(expr) => expr.compile(scope),
            BinaryOperator::LessEqual(expr) => expr.compile(scope),
            BinaryOperator::GreaterThan(expr) => expr.compile(scope),
            BinaryOperator::GreaterEqual(expr) => expr.compile(scope),
            BinaryOperator::Equals(expr) => expr.compile(scope),
            BinaryOperator::NotEqual(expr) => expr.compile(scope),
            BinaryOperator::And(expr) => expr.compile(scope),
            BinaryOperator::Or(expr) => expr.compile(scope),
        }
    }
}
