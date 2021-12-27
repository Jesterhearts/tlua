use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use thiserror::Error;

use crate::{
    ast::{
        block::Block,
        constant_string::ConstantString,
        expressions::{
            function_defs::*,
            operator::*,
            tables::*,
            *,
        },
        identifiers::Ident,
        prefix_expression::*,
        statement::{
            assignment::*,
            fn_decl::*,
            for_loop::*,
            foreach_loop::*,
            if_statement::*,
            repeat_loop::*,
            variables::*,
            while_loop::*,
            *,
        },
    },
    values::*,
    vm::{
        opcodes::Instruction,
        Constant,
        FuncId,
        Number,
        OpError,
    },
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
pub(crate) enum NodeOutput {
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

#[enum_dispatch(BinaryOperator, UnaryOperator, Expression)]
pub(crate) trait CompileExpression {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError>;
}

#[enum_dispatch(Statement)]
pub(crate) trait CompileStatement {
    // TODO(compiler-opt): For e.g. if statements, the compiler could use knowledge
    // of ret statements to omit instructions.
    // This would require changing the result of this to an enum of:
    //      { Raise(OpError), Return }
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError>;
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
pub(crate) struct Function {
    pub(crate) named_args: usize,
    pub(crate) local_registers: usize,
    pub(crate) anon_registers: usize,
    pub(crate) instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub(crate) globals_map: HashMap<Ident, usize>,
    pub(crate) functions: Vec<Function>,
    pub(crate) main: Function,
}
