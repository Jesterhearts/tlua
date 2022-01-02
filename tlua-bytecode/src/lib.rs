use std::{
    fmt::Debug,
    ops::Deref,
};

use thiserror::Error;

pub mod binop;
mod constant;
mod number;
pub mod opcodes;
mod register;

pub use constant::Constant;
pub use number::Number;
pub use register::Register;

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum ByteCodeError {
    // TODO(ergo): include instruction information
    #[error("Call setup instruction encountered outside of a call context")]
    UnexpectedCallInstruction,
    #[error("Non call setup instruction encountered inside of a call context")]
    ExpectedCallInstruction,
    #[error("Expected a *DoCall instruction.")]
    MissingCallInvocation,
    #[error("Expected a jump instruction")]
    MissingJump,
    #[error("Expected a scope descriptor")]
    MissingScopeDescriptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Error)]
pub enum OpError {
    #[error("Invalid types for operator {op:?}")]
    InvalidType { op: &'static str },
    #[error("Cannot index a numeric value")]
    IndexNumberErr,
    #[error("Cannot index a boolean value")]
    IndexBoolErr,
    #[error("Cannot index a nil value")]
    IndexNilErr,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncId(pub usize);

impl Deref for FuncId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
