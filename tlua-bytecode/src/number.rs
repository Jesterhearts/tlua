use std::hash::{
    Hash,
    Hasher,
};

use tlua_parser::ast;

use crate::{
    binop::f64inbounds,
    NumLike,
};

#[derive(Debug, Clone, Copy)]
pub enum Number {
    Float(f64),
    Integer(i64),
}

impl Number {
    /// Hashes the number.
    ///
    /// # Warning
    /// You may not rely on equal hash values implying equal values. i.e. the
    /// following may panic:
    /// ```ignore
    /// if Number::hash(a) == Number::hash(b) {
    ///     assert!(a == b);
    /// }
    /// ```
    pub fn hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        self.hash_into(&mut hasher);

        hasher.finish()
    }

    /// Hashes the number using the provided hasher.
    ///
    /// # Warning
    /// You may not rely on equal hash values implying equal values. i.e. the
    /// following may panic:
    /// ```ignore
    /// if Number::hash_into(a, &mut hasher) == Number::hash_into(b, &mut hasher) {
    ///     assert!(a == b);
    /// }
    /// ```
    pub fn hash_into(&self, hasher: &mut impl Hasher) {
        match *self {
            Number::Float(f) => {
                if f.is_nan() {
                    std::mem::discriminant(self).hash(hasher)
                } else if let Ok(i) = f64inbounds(f) {
                    std::mem::discriminant(&Number::Integer(i)).hash(hasher);
                    i.hash(hasher)
                } else {
                    std::mem::discriminant(self).hash(hasher);
                    f.to_bits().hash(hasher)
                }
            }
            Number::Integer(i) => {
                std::mem::discriminant(self).hash(hasher);
                i.hash(hasher)
            }
        }
    }
}

impl From<ast::expressions::number::Number> for Number {
    fn from(ast_num: ast::expressions::number::Number) -> Self {
        match ast_num {
            ast::expressions::number::Number::Float(f) => Self::Float(f),
            ast::expressions::number::Number::Integer(i) => Self::Integer(i),
        }
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::Float(l0), Number::Float(r0)) => l0 == r0,
            (Number::Integer(l0), Number::Integer(r0)) => l0 == r0,
            (Number::Float(l0), Number::Integer(r0)) => *l0 == *r0 as f64,
            (Number::Integer(l0), Number::Float(r0)) => *l0 as f64 == *r0,
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Number::Float(l0), Number::Float(r0)) => l0.partial_cmp(r0),
            (Number::Integer(l0), Number::Integer(r0)) => l0.partial_cmp(r0),
            (Number::Float(l0), Number::Integer(r0)) => l0.partial_cmp(&(*r0 as f64)),
            (Number::Integer(l0), Number::Float(r0)) => (*l0 as f64).partial_cmp(r0),
        }
    }
}

impl NumLike for &'_ Number {
    fn as_float(&self) -> Option<f64> {
        match self {
            Number::Float(f) => Some(*f),
            Number::Integer(i) => Some(*i as f64),
        }
    }

    fn as_int(&self) -> Option<i64> {
        if let Number::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }
}
