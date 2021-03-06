use std::{
    collections::HashMap,
    num::NonZeroUsize,
};

use derive_more::{
    Deref,
    From,
    Into,
};
use scopeguard::guard_on_success;
use thiserror::Error;
use tlua_bytecode::{
    opcodes::Instruction,
    Constant,
    ImmediateRegister,
    OpError,
    TypeId,
};
use tlua_parser::{
    expressions::Expression,
    identifiers::Ident,
    parse_chunk,
    statement::Statement,
    ASTAllocator,
    ChunkParseError,
    StringTable,
};

mod block;
mod compiler;
mod expressions;
mod prefix_expression;
mod statement;

use self::compiler::Scope;
use crate::compiler::{
    unasm::MappedLocalRegister,
    Compiler,
    RegisterOps,
};

#[derive(Debug)]
enum NodeOutput {
    Constant(Constant),
    Immediate(ImmediateRegister),
    MappedRegister(MappedLocalRegister),
    TableEntry {
        table: ImmediateRegister,
        index: ImmediateRegister,
    },
    ReturnValues,
    VAStack,
    Err(OpError),
}

impl NodeOutput {
    pub(crate) fn into_register(self, scope: &mut Scope) -> ImmediateRegister {
        match self {
            NodeOutput::Constant(c) => {
                let reg = scope.push_immediate();
                reg.set_from_constant(scope, c).unwrap();
                reg
            }
            NodeOutput::Immediate(i) => i,
            NodeOutput::MappedRegister(other) => {
                let reg = scope.push_immediate();
                reg.set_from_local(scope, other).unwrap();
                reg
            }
            NodeOutput::TableEntry { table, index } => {
                let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(index));
                table
                    .set_from_table_entry(&mut scope, table, index)
                    .unwrap();
                table
            }
            NodeOutput::ReturnValues => {
                let reg = scope.push_immediate();
                reg.set_from_ret(scope).unwrap();
                reg
            }
            NodeOutput::VAStack => {
                let reg = scope.push_immediate();
                reg.set_from_va(scope, 0).unwrap();
                reg
            }
            NodeOutput::Err(_) => scope.push_immediate(),
        }
    }

    pub(crate) fn into_existing_register(self, scope: &mut Scope, dest: ImmediateRegister) {
        match self {
            NodeOutput::Constant(value) => dest.set_from_constant(scope, value).unwrap(),
            NodeOutput::Immediate(other) => {
                let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(other));
                dest.set_from_immediate(&mut scope, other).unwrap()
            }
            NodeOutput::MappedRegister(other) => dest.set_from_local(scope, other).unwrap(),
            NodeOutput::TableEntry { table, index } => {
                dest.set_from_table_entry(scope, table, index).unwrap()
            }
            NodeOutput::ReturnValues => dest.set_from_ret(scope).unwrap(),
            NodeOutput::VAStack => dest.set_from_va(scope, 0).unwrap(),
            NodeOutput::Err(_) => (),
        };
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

#[derive(Debug)]
pub(crate) enum Void {}

impl From<Void> for CompileError {
    fn from(_: Void) -> Self {
        unreachable!()
    }
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
    pub immediates: usize,
    pub instructions: Instructions,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub strings: StringTable,
    pub globals_map: HashMap<Ident, usize>,
    pub functions: Vec<Function>,
    pub main: Function,
}

pub fn compile(src: &str) -> Result<Chunk, CompileError> {
    let alloc = ASTAllocator::default();
    let mut strings = StringTable::default();

    let ast = parse_chunk(src, &alloc, &mut strings).map_err(CompileError::ParseError)?;

    Compiler::new(strings).compile_ast(ast)
}
