use std::{
    cell::RefCell,
    rc::Rc,
};

use derive_more::{
    Deref,
    DerefMut,
    From,
    Into,
};
use tracing_rc::{
    rc::Trace,
    Trace,
};

use crate::vm::{
    opcodes::ScopeDescriptor,
    FuncId,
    Register,
    Value,
};

#[derive(Debug, Default, Clone)]
pub(crate) struct Scope {
    pub(crate) registers: Rc<Vec<RefCell<Value>>>,
}

impl Scope {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            registers: Rc::new(vec![RefCell::new(Value::Nil); size]),
        }
    }
}

impl Trace for Scope {
    fn visit_children(&self, visitor: &mut tracing_rc::rc::GcVisitor) {
        for v in self.registers.iter() {
            v.visit_children(visitor);
        }
    }
}

impl std::hash::Hash for Scope {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.registers.as_ptr(), state)
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.registers.as_ptr(), other.registers.as_ptr())
    }
}

impl Eq for Scope {}

#[derive(Debug, Default, Deref, DerefMut, From, Into)]
pub(crate) struct Results(Vec<Value>);

#[derive(Debug, Default, Deref, DerefMut, From, Into)]
pub(crate) struct VaArgs(Vec<Value>);

pub(crate) struct ScopeSet {
    referenced: Vec<Scope>,
    local: Scope,
    anon: Vec<Value>,

    va_args: VaArgs,
    results: Results,
}

impl ScopeSet {
    pub(crate) fn new(
        // TODO(perf): This might benefit from being COW
        referenced: Vec<Scope>,
        local: Scope,
        anon: Vec<Value>,
        va_args: Vec<Value>,
    ) -> ScopeSet {
        ScopeSet {
            referenced,
            local,
            anon,
            va_args: va_args.into(),
            results: Default::default(),
        }
    }

    pub(crate) fn into_results(self) -> Results {
        self.results
    }

    pub(crate) fn into_results_and_va(self) -> (Results, VaArgs) {
        (self.results, self.va_args)
    }

    pub(crate) fn push_scope(&mut self, descriptor: ScopeDescriptor) {
        self.referenced.push(self.local.clone());
        self.local = Scope::new(descriptor.size);
    }

    pub(crate) fn pop_scope(&mut self) {
        self.local = self
            .referenced
            .pop()
            .expect("Pop should always come after push");
    }

    pub(crate) fn load_va(&self, index: usize) -> Value {
        self.va_args.get(index).cloned().unwrap_or(Value::Nil)
    }

    pub(crate) fn iter_va(&self) -> impl Iterator<Item = &Value> + '_ {
        self.va_args.iter()
    }

    pub(crate) fn add_result(&mut self, val: Value) {
        self.results.push(val);
    }

    pub(crate) fn extend_results(&mut self, other: impl IntoIterator<Item = Value>) {
        self.results.extend(other.into_iter());
    }

    #[track_caller]
    pub(crate) fn load(&self, addr: Register) -> Value {
        if let Some(scope) = addr.scope {
            if usize::from(scope.get() - 1) == self.referenced.len() {
                self.local.registers[usize::from(addr.offset)]
                    .borrow()
                    .clone()
            } else {
                self.referenced[usize::from(scope.get()) - 1].registers[usize::from(addr.offset)]
                    .borrow()
                    .clone()
            }
        } else {
            self.anon[usize::from(addr.offset)].clone()
        }
    }

    #[track_caller]
    pub(crate) fn store(&mut self, addr: Register, value: Value) {
        if let Some(scope) = addr.scope {
            if usize::from(scope.get() - 1) == self.referenced.len() {
                self.local.registers[usize::from(addr.offset)].replace(value);
            } else {
                self.referenced[usize::from(scope.get()) - 1].registers[usize::from(addr.offset)]
                    .replace(value);
            }
        } else {
            self.anon[usize::from(addr.offset)] = value;
        }
    }

    #[track_caller]
    pub(crate) fn copy(&mut self, dest: Register, src: Register) {
        let src_data = self.load(src);
        self.store(dest, src_data);
    }
}

#[derive(Debug, Trace)]
pub struct Function {
    pub(crate) referenced_scopes: Vec<Scope>,

    #[trace(ignore)]
    pub(crate) id: FuncId,
}

impl Function {
    pub(crate) fn new(available_scope: &ScopeSet, id: FuncId) -> Self {
        // TODO(perf): This is way too pessimistic and could use info from the compiler
        // to cut down on the size of the scopes it captures.
        let mut referenced_scopes = available_scope.referenced.clone();
        referenced_scopes.extend(std::iter::once(available_scope.local.clone()));
        Self {
            referenced_scopes,
            id,
        }
    }
}
