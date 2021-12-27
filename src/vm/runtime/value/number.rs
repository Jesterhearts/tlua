pub(crate) trait NumLike {
    fn as_float(&self) -> Option<f64>;
    fn as_int(&self) -> Option<i64>;
}

#[derive(Debug, Clone, Copy)]
pub enum Number {
    Float(f64),
    Integer(i64),
}

impl std::hash::Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match *self {
            Number::Float(f) => {
                // Hash is used for indexing tables.
                debug_assert!(!f.is_nan());

                if f.fract() == 0.0 && f > i64::MIN as f64 && f < i64::MIN as f64 {
                    (f as i64).hash(state)
                } else {
                    f.to_bits().hash(state)
                }
            }
            Number::Integer(i) => i.hash(state),
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

impl NumLike for Number {
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
