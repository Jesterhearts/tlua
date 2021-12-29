use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
};

use derive_more::From;
pub use tlua_bytecode::Number;
use tlua_bytecode::{
    Constant,
    NumLike,
    Truthy,
};
use tracing_rc::{
    rc::Gc,
    Trace,
};

use crate::values::LuaString;

pub mod function;
pub mod table;

pub use self::{
    function::Function,
    table::Table,
};

#[derive(Debug, Clone, Trace, From)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(Number),
    String(Rc<RefCell<LuaString>>),
    Table(#[trace] Gc<Table>),
    Function(#[trace] Gc<Function>),
    Userdata(((),)),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Table(_), Self::Table(_)) => todo!(),
            (Self::Userdata(_), Self::Userdata(_)) => todo!(),
            (Self::Function(_), Self::Function(_)) => todo!(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl From<Constant> for Value {
    fn from(c: Constant) -> Self {
        match c {
            Constant::Nil => Self::Nil,
            Constant::Bool(b) => b.into(),
            Constant::Float(f) => f.into(),
            Constant::Integer(i) => i.into(),
            Constant::String(s) => Self::String(Rc::new(RefCell::new(s.into()))),
        }
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Number(Number::Integer(i))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::Number(Number::Float(f))
    }
}

impl Truthy for Value {
    fn as_bool(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl NumLike for Value {
    fn as_float(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_float(),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_int(),
            _ => None,
        }
    }
}
