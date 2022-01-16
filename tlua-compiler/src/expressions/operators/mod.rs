use tlua_parser::ast::expressions::operator::{
    BinaryOperator,
    UnaryOperator,
};

use crate::{
    compiler::CompilerContext,
    CompileError,
    CompileExpression,
    NodeOutput,
};

pub(crate) mod binop;
pub(crate) mod unop;

impl CompileExpression for UnaryOperator<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            UnaryOperator::Minus(expr) => expr.compile(compiler),
            UnaryOperator::Not(expr) => expr.compile(compiler),
            UnaryOperator::Length(expr) => expr.compile(compiler),
            UnaryOperator::BitNot(expr) => expr.compile(compiler),
        }
    }
}

impl CompileExpression for BinaryOperator<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            BinaryOperator::Plus(expr) => expr.compile(compiler),
            BinaryOperator::Minus(expr) => expr.compile(compiler),
            BinaryOperator::Times(expr) => expr.compile(compiler),
            BinaryOperator::Divide(expr) => expr.compile(compiler),
            BinaryOperator::IDiv(expr) => expr.compile(compiler),
            BinaryOperator::Modulo(expr) => expr.compile(compiler),
            BinaryOperator::Exponetiation(expr) => expr.compile(compiler),
            BinaryOperator::BitAnd(expr) => expr.compile(compiler),
            BinaryOperator::BitOr(expr) => expr.compile(compiler),
            BinaryOperator::BitXor(expr) => expr.compile(compiler),
            BinaryOperator::ShiftLeft(expr) => expr.compile(compiler),
            BinaryOperator::ShiftRight(expr) => expr.compile(compiler),
            BinaryOperator::Concat(expr) => expr.compile(compiler),
            BinaryOperator::LessThan(expr) => expr.compile(compiler),
            BinaryOperator::LessEqual(expr) => expr.compile(compiler),
            BinaryOperator::GreaterThan(expr) => expr.compile(compiler),
            BinaryOperator::GreaterEqual(expr) => expr.compile(compiler),
            BinaryOperator::Equals(expr) => expr.compile(compiler),
            BinaryOperator::NotEqual(expr) => expr.compile(compiler),
            BinaryOperator::And(expr) => expr.compile(compiler),
            BinaryOperator::Or(expr) => expr.compile(compiler),
        }
    }
}
