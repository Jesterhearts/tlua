use std::{
    cell::RefCell,
    fmt::Debug,
    hash::{
        Hash,
        Hasher,
    },
    rc::Rc,
};

use derive_more::From;
pub use tlua_bytecode::Number;
use tlua_bytecode::{
    opcodes::{
        AnyReg,
        Operand,
    },
    NumLike,
    Register,
    Truthy,
};
use tracing_rc::{
    rc::Gc,
    Trace,
};

pub mod function;
pub mod string;
pub mod table;

pub use self::{
    function::Function,
    string::LuaString,
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

impl Value {
    /// Hashes the value.
    ///
    /// # Warning
    /// You may not rely on equal hash values implying equal values. i.e. the
    /// following may panic:
    /// ```ignore
    /// if Value::hash(a) == Value::hash(b) {
    ///     assert!(a == b);
    /// }
    /// ```
    pub fn hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        self.hash_into(&mut hasher);
        hasher.finish()
    }

    /// Hashes the value using the provided hasher.
    ///
    /// # Warning
    /// You may not rely on equal hash values implying equal values. i.e. the
    /// following may panic:
    /// ```ignore
    /// if Value::hash_into(a, &mut hasher) == Value::hash_into(b, &mut hasher) {
    ///     assert!(a == b);
    /// }
    /// ```
    pub fn hash_into(&self, hasher: &mut impl Hasher) {
        std::mem::discriminant(self).hash(hasher);

        match self {
            Value::Nil => (),
            Value::Bool(b) => b.hash(hasher),
            Value::Number(n) => n.hash_into(hasher),
            Value::String(s) => s.borrow().hash(hasher),
            Value::Table(t) => std::ptr::hash(&*t.borrow(), hasher),
            Value::Function(f) => f.borrow().hash(hasher),
            Value::Userdata(_) => todo!(),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Table(l0), Self::Table(r0)) => l0 == r0,
            (Self::Userdata(_), Self::Userdata(_)) => todo!(),
            (Self::Function(_), Self::Function(_)) => todo!(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl TryFrom<Operand<Register>> for Value {
    type Error = AnyReg<Register>;

    fn try_from(value: Operand<Register>) -> Result<Self, Self::Error> {
        match value {
            Operand::Nil => Ok(Self::Nil),
            Operand::Bool(b) => Ok(b.into()),
            Operand::Float(f) => Ok(f.into()),
            Operand::Integer(i) => Ok(i.into()),
            Operand::String(s) => Ok(Self::String(Rc::new(RefCell::new(s.into())))),
            Operand::Register(r) => Err(r.into()),
            Operand::Immediate(i) => Err(i.into()),
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(Rc::new(RefCell::new(s.into())))
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
