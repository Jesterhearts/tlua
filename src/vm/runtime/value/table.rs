use std::{
    cell::RefCell,
    collections::HashMap,
};

use crate::vm::{
    runtime::{
        heap::Traceable,
        GcVisitor,
    },
    Value,
};

#[derive(Debug, Default)]
pub struct Table {
    pub(crate) entries: HashMap<Value, Value>,
}

impl Traceable for RefCell<Table> {
    unsafe fn visit_children(&self, visitor: &mut GcVisitor) {
        let this = self.borrow();

        for v in this.entries.keys().chain(this.entries.values()) {
            v.visit_children(visitor);
        }
    }
}
