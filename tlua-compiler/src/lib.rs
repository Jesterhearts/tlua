use std::{
    collections::HashMap,
    num::NonZeroUsize,
};

use derive_more::{
    Deref,
    From,
    Into,
};
use thiserror::Error;
use tlua_bytecode::{
    opcodes::Instruction,
    AnonymousRegister,
    OpError,
    TypeId,
};
use tlua_parser::{
    ast::{
        expressions::Expression,
        identifiers::Ident,
        statement::Statement,
        ASTAllocator,
    },
    parsing::{
        parse_chunk,
        ChunkParseError,
    },
};

mod block;
mod compiler;
mod constant;
mod expressions;
mod prefix_expression;
mod statement;

use self::compiler::Scope;
use crate::{
    compiler::{
        unasm::MappedLocalRegister,
        Compiler,
    },
    constant::Constant,
};

#[derive(Debug, PartialEq, Clone, Copy)]
enum NodeOutput {
    Constant(Constant),
    Immediate(AnonymousRegister),
    MappedRegister(MappedLocalRegister),
    TableEntry {
        table: AnonymousRegister,
        index: AnonymousRegister,
    },
    ReturnValues,
    VAStack,
    Err(OpError),
}

impl Default for NodeOutput {
    fn default() -> Self {
        NodeOutput::Constant(Constant::Nil)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, From, Into)]
pub struct FuncId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    Table,
    Function(FuncId),
}

impl From<BuiltinType> for TypeId {
    fn from(id: BuiltinType) -> Self {
        match id {
            BuiltinType::Function(id) => Self::Any(NonZeroUsize::new(usize::from(id) + 2).unwrap()),
            BuiltinType::Table => Self::Any(NonZeroUsize::new(1).unwrap()),
        }
    }
}

impl TryFrom<TypeId> for BuiltinType {
    type Error = ();

    fn try_from(value: TypeId) -> Result<Self, Self::Error> {
        match value {
            TypeId::Primitive(_) => Err(()),
            TypeId::Any(type_id) => match type_id.get() {
                1 => Ok(Self::Table),
                id => Ok(Self::Function(FuncId::from(id - 2))),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Error parsing lua source: {0:?}")]
    ParseError(ChunkParseError),
    #[error("Duplicate label: {label}")]
    DuplicateLabel { label: String },
    #[error("Goto {label} jumps into scope of local")]
    JumpIntoLocalScope { label: String },
    #[error("Cannot use ... outside of a vararg function")]
    NoVarArgsAvailable,
    #[error("Allocated globals exceeded the maximum of {max:}")]
    TooManyGlobals { max: usize },
    #[error("Allocated locals exceeded the maximum of {max:}")]
    TooManyLocals { max: usize },
    #[error("The level of scope nesting has exceeded the maximum depth of {max:}")]
    ScopeNestingTooDeep { max: usize },
    #[error("The specified table index exceeds the max entries.")]
    TooManyTableEntries { max: usize },
}

trait CompileExpression {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError>;
}

impl CompileExpression for Expression<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        match self {
            Expression::Parenthesized(expr) => expr.compile(scope),
            Expression::Variable(expr) => expr.compile(scope),
            Expression::FunctionCall(expr) => CompileExpression::compile(expr, scope),
            Expression::Nil(expr) => expr.compile(scope),
            Expression::Bool(expr) => expr.compile(scope),
            Expression::Number(expr) => expr.compile(scope),
            Expression::String(expr) => expr.compile(scope),
            Expression::FnDef(expr) => expr.compile(scope),
            Expression::TableConstructor(expr) => expr.compile(scope),
            Expression::VarArgs(expr) => expr.compile(scope),
            Expression::BinaryOp(expr) => expr.compile(scope),
            Expression::UnaryOp(expr) => expr.compile(scope),
        }
    }
}

trait CompileStatement {
    // TODO(compiler-opt): For e.g. if statements, the compiler could use knowledge
    // of ret statements to omit instructions.
    // This would require changing the result of this to an enum of:
    //      { Raise(OpError), Return }
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError>;
}

impl CompileStatement for Statement<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        match self {
            Statement::Empty(stat) => stat.compile(scope),
            Statement::Assignment(stat) => stat.compile(scope),
            Statement::Call(stat) => CompileStatement::compile(stat, scope),
            Statement::Label(stat) => stat.compile(scope),
            Statement::Break(stat) => stat.compile(scope),
            Statement::Goto(stat) => stat.compile(scope),
            Statement::Do(stat) => stat.compile(scope),
            Statement::While(stat) => stat.compile(scope),
            Statement::Repeat(stat) => stat.compile(scope),
            Statement::If(stat) => stat.compile(scope),
            Statement::For(stat) => stat.compile(scope),
            Statement::ForEach(stat) => stat.compile(scope),
            Statement::FnDecl(stat) => stat.compile(scope),
            Statement::LocalVarList(stat) => stat.compile(scope),
        }
    }
}

impl<T> CompileExpression for &'_ T
where
    T: CompileExpression,
{
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        (*self).compile(scope)
    }
}

impl<T> CompileStatement for &'_ T
where
    T: CompileStatement,
{
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        (*self).compile(scope)
    }
}

#[derive(Clone, Deref, From)]
pub struct Instructions(Vec<Instruction>);

impl std::fmt::Debug for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        for (idx, op) in self.0.iter().enumerate() {
            list.entry(&format_args!("{:4}: {:?}", idx, op));
        }
        list.finish()
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub named_args: usize,
    pub local_registers: usize,
    pub anon_registers: usize,
    pub instructions: Instructions,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub globals_map: HashMap<Ident, usize>,
    pub functions: Vec<Function>,
    pub main: Function,
}

pub fn compile(src: &str) -> Result<Chunk, CompileError> {
    let alloc = ASTAllocator::default();

    let ast = parse_chunk(src, &alloc).map_err(CompileError::ParseError)?;

    Compiler::default().compile_ast(ast)
}
