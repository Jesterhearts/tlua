use std::collections::HashMap;

use thiserror::Error;
use tlua_parser::ast::{
    expressions::{
        operator::{
            BinaryOperator,
            UnaryOperator,
        },
        Expression,
    },
    identifiers::Ident,
    statement::Statement,
};

use crate::vm::{
    opcodes::Instruction,
    Constant,
    FuncId,
    OpError,
};

pub mod block;
pub mod compiler;
pub mod expressions;
pub mod prefix_expression;
pub mod statement;

use self::compiler::{
    unasm::UnasmRegister,
    CompilerContext,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeOutput {
    Function(FuncId),
    Constant(Constant),
    Register(UnasmRegister),
    ReturnValues,
    VAStack,
    Err(OpError),
}

impl Default for NodeOutput {
    fn default() -> Self {
        NodeOutput::Constant(Constant::Nil)
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq)]
pub enum CompileError {
    #[error("Cannot use ... outside of a vararg function")]
    NoVarArgsAvailable,
    #[error("Allocated globals exceeded the maximum of {max:}")]
    TooManyGlobals { max: usize },
    #[error("Allocated locals exceeded the maximum of {max:}")]
    TooManyLocals { max: usize },
    #[error("The level of scope nesting has exceeded the maximum depth of {max:}")]
    ScopeNestingTooDeep { max: usize },
}

pub trait CompileExpression {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError>;
}

impl CompileExpression for Expression<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        match self {
            Expression::Parenthesized(expr) => expr.compile(compiler),
            Expression::Variable(expr) => expr.compile(compiler),
            Expression::FunctionCall(expr) => CompileExpression::compile(expr, compiler),
            Expression::Nil(expr) => expr.compile(compiler),
            Expression::Bool(expr) => expr.compile(compiler),
            Expression::Number(expr) => expr.compile(compiler),
            Expression::String(expr) => expr.compile(compiler),
            Expression::FnDef(expr) => expr.compile(compiler),
            Expression::TableConstructor(expr) => expr.compile(compiler),
            Expression::VarArgs(expr) => expr.compile(compiler),
            Expression::BinaryOp(expr) => expr.compile(compiler),
            Expression::UnaryOp(expr) => expr.compile(compiler),
        }
    }
}

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

pub trait CompileStatement {
    // TODO(compiler-opt): For e.g. if statements, the compiler could use knowledge
    // of ret statements to omit instructions.
    // This would require changing the result of this to an enum of:
    //      { Raise(OpError), Return }
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError>;
}

impl CompileStatement for Statement<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        match self {
            Statement::Empty(stat) => stat.compile(compiler),
            Statement::Assignment(stat) => stat.compile(compiler),
            Statement::Call(stat) => CompileStatement::compile(stat, compiler),
            Statement::Label(stat) => stat.compile(compiler),
            Statement::Break(stat) => stat.compile(compiler),
            Statement::Goto(stat) => stat.compile(compiler),
            Statement::Do(stat) => stat.compile(compiler),
            Statement::While(stat) => stat.compile(compiler),
            Statement::Repeat(stat) => stat.compile(compiler),
            Statement::If(stat) => stat.compile(compiler),
            Statement::For(stat) => stat.compile(compiler),
            Statement::ForEach(stat) => stat.compile(compiler),
            Statement::FnDecl(stat) => stat.compile(compiler),
            Statement::LocalVarList(stat) => stat.compile(compiler),
        }
    }
}

impl<T> CompileExpression for &'_ T
where
    T: CompileExpression,
{
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        (*self).compile(compiler)
    }
}

impl<T> CompileStatement for &'_ T
where
    T: CompileStatement,
{
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        (*self).compile(compiler)
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub named_args: usize,
    pub local_registers: usize,
    pub anon_registers: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub globals_map: HashMap<Ident, usize>,
    pub functions: Vec<Function>,
    pub main: Function,
}
