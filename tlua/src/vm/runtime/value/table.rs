use std::collections::HashMap;

use tlua_bytecode::{
    Number,
    OpError,
};
use tracing_rc::Trace;

use crate::vm::runtime::Value;

#[derive(Debug, Default, Trace)]
pub struct Table {
    pub entries: HashMap<TableKey, Value>,
}

#[derive(Debug, Clone, Trace)]
pub struct TableKey(Value);

impl PartialEq for TableKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// We validate in TryFrom that no NaNs exist.
impl Eq for TableKey {}

impl TryFrom<Value> for TableKey {
    type Error = OpError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(Number::Float(f)) => {
                if f.is_nan() {
                    Err(OpError::TableIndexNaN)
                } else {
                    Ok(Self(Value::Number(Number::Float(f))))
                }
            }
            v @ Value::Number(Number::Integer(_)) | v => Ok(Self(v)),
        }
    }
}

// We validate in TryFrom that no NaNs exist.
impl std::hash::Hash for TableKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash_into(state)
    }
}
