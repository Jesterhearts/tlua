use std::collections::HashMap;

use tracing_rc::Trace;

use crate::vm::Value;

#[derive(Debug, Default, Trace)]
pub struct Table {
    pub entries: HashMap<Value, Value>,
}
