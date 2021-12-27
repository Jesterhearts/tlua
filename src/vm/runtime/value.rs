use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
};

use derive_more::From;

use crate::{
    values::LuaString,
    vm::{
        runtime::{
            heap::{
                GcPtr,
                Traceable,
            },
            GcVisitor,
        },
        Constant,
    },
};

pub(crate) mod function;
pub(crate) mod number;
pub(crate) mod table;

pub(crate) use self::number::NumLike;
pub use self::{
    function::Function,
    number::Number,
    table::Table,
};

#[derive(Debug, Clone, From)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(Number),
    String(Rc<RefCell<LuaString>>),
    Table(GcPtr<RefCell<Table>>),
    Userdata(((),)),
    Function(GcPtr<RefCell<Function>>),
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

impl Traceable for Value {
    unsafe fn visit_children(&self, visitor: &mut GcVisitor) {
        match self {
            Value::Table(t) => visitor(t.node()),
            Value::Function(f) => visitor(f.node()),

            Value::Userdata(_) => todo!(),

            Value::Nil | Value::Bool(_) | Value::Number(_) | Value::String(_) => (),
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

impl Value {
    pub(crate) fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Nil => false,
            _ => true,
        }
    }
}
