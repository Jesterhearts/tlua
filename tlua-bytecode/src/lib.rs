use std::{
    fmt::Debug,
    num::NonZeroUsize,
};

use derive_more::{
    From,
    Into,
};
use thiserror::Error;

pub mod binop;
mod number;
pub mod opcodes;
mod register;

pub use number::Number;
pub use register::{
    AnonymousRegister,
    MappedRegister,
    Register,
};
use tlua_parser::ast::constant_string::ConstantString;

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum ByteCodeError {
    // TODO(ergo): include instruction information
    #[error("Call setup instruction encountered outside of a call context")]
    UnexpectedCallInstruction,
    #[error("Non call setup instruction encountered inside of a call context")]
    ExpectedArgMappingInstruction,
    #[error("Non return value mapping instruction encountered during function cleanup")]
    ExpectedReturnValueInstruction,
    #[error("Expected a *DoCall instruction.")]
    MissingCallInvocation,
    #[error("Expected a jump instruction")]
    MissingJump,
    #[error("Expected a scope descriptor")]
    MissingScopeDescriptor,
    #[error("Invalid type metadata")]
    InvalidTypeMetadata,
    #[error("Invalid type id")]
    InvalidTypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum OpError {
    #[error("Invalid types for operator {op:?}")]
    InvalidType { op: &'static str },
    #[error("Attempted to index a {ty} value")]
    NotATable { ty: &'static str },
    #[error("Attempted to compare {lhs} with {rhs}")]
    CmpErr {
        lhs: &'static str,
        rhs: &'static str,
    },
    #[error("Attempted to compare two {type_name} values")]
    DuoCmpErr { type_name: &'static str },
    #[error("Float {f:?} cannot be converted to int")]
    FloatToIntConversionFailed { f: f64 },
    #[error("Table index is NaN")]
    TableIndexNaN,
    #[error("Table index out of bounds")]
    TableIndexOutOfBounds,
    #[error("Meta method {name} not found")]
    NoSuchMetaMethod { name: &'static str },
    #[error("Miscompiled bytecode ({err}) at offset {offset} in sequence")]
    ByteCodeError { err: ByteCodeError, offset: usize },
}

pub trait StringLike {
    fn as_bytes(&self) -> &[u8];
}

pub trait NumLike {
    fn as_float(&self) -> Option<f64>;
    fn as_int(&self) -> Option<i64>;
}

pub trait Truthy {
    fn as_bool(&self) -> bool;
}

impl StringLike for ConstantString {
    fn as_bytes(&self) -> &[u8] {
        self.data().as_slice()
    }
}

/// A type identifier used for bytecodes like `Alloc`. The exact meaning is up
/// to the runtime/compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct TypeId(usize);

impl TypeId {
    pub const fn const_from(v: usize) -> Self {
        Self(v)
    }
}

/// Extended type information used for bytecodes. The exact meaning is up to the
/// runtime/compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct TypeMeta(Option<NonZeroUsize>);
