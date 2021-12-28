use tlua_parser::ast::constant_string::ConstantString;

use crate::vm::runtime::value::{
    NumLike,
    Number,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constant {
    Nil,
    Bool(bool),
    Float(f64),
    Integer(i64),
    String(ConstantString),
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
        todo!()
        // Value::from(*self).short_type_name()
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Constant::Nil => false,
            Constant::Bool(b) => *b,
            _ => true,
        }
    }
}
