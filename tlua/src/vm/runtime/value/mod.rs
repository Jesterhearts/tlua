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
    NumLike,
    Truthy,
};
use tlua_strings::LuaString;
use tracing_rc::{
    rc::Gc,
    Trace,
};

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
            (Self::Function(l0), Self::Function(r0)) => l0.borrow().id == r0.borrow().id,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl From<LuaString> for Value {
    fn from(s: LuaString) -> Self {
        Self::String(Rc::new(RefCell::new(s)))
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

impl Truthy for &'_ Value {
    fn as_bool(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl Truthy for Value {
    fn as_bool(&self) -> bool {
        (&self).as_bool()
    }
}

impl NumLike for &'_ Value {
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
