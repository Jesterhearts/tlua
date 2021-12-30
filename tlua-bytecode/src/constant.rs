pub use tlua_parser::ast::constant_string::ConstantString;

use crate::{
    NumLike,
    Number,
    StringLike,
    Truthy,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constant {
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(ConstantString),
}

impl From<f64> for Constant {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<i64> for Constant {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<Number> for Constant {
    fn from(n: Number) -> Self {
        match n {
            Number::Float(f) => Self::Float(f),
            Number::Integer(i) => Self::Integer(i),
        }
    }
}

impl From<bool> for Constant {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Constant> for bool {
    fn from(c: Constant) -> Self {
        c.as_bool()
    }
}

impl Truthy for Constant {
    fn as_bool(&self) -> bool {
        match self {
            Constant::Nil => false,
            Constant::Bool(b) => *b,
            _ => true,
        }
    }
}

impl NumLike for Constant {
    fn as_float(&self) -> Option<f64> {
        match self {
            Constant::Float(f) => Some(*f),
            Constant::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_int(&self) -> Option<i64> {
        match self {
            Constant::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

impl Constant {
    pub fn short_type_name(&self) -> &'static str {
        match self {
            Constant::Nil => "nil",
            Constant::Bool(_) => "bool",
            Constant::Float(_) | Constant::Integer(_) => "number",
            Constant::String(_) => "string",
        }
    }
}

impl StringLike for ConstantString {
    fn as_bytes(&self) -> &[u8] {
        self.data().as_slice()
    }
}
