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
use tlua_bytecode::{
    opcodes::ScopeDescriptor,
    MappedRegister,
    Register,
};
use tlua_compiler::FuncId;
use tracing_rc::{
    rc::Trace,
    Trace,
};

use crate::vm::runtime::Value;

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub registers: Rc<Vec<RefCell<Value>>>,
}

impl Scope {
    pub fn new(size: usize) -> Self {
        Self {
            registers: Rc::new(vec![RefCell::new(Value::Nil); size]),
        }
    }

    pub fn into_values(mut self) -> Vec<RefCell<Value>> {
        Rc::make_mut(&mut self.registers).drain(..).collect()
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
pub struct Results(Vec<Value>);

#[derive(Debug, Default, Deref, DerefMut, From, Into)]
pub struct VaArgs(Vec<Value>);

#[derive(Debug)]
pub struct ScopeSet {
    referenced: Vec<Scope>,
    local: Scope,

    va_args: VaArgs,
    results: Results,
}

impl ScopeSet {
    pub fn new(
        // TODO(perf): This might benefit from being COW
        referenced: Vec<Scope>,
        local: Scope,
        va_args: Vec<Value>,
    ) -> ScopeSet {
        ScopeSet {
            referenced,
            local,
            va_args: va_args.into(),
            results: Default::default(),
        }
    }

    pub fn into_results(self) -> Results {
        self.results
    }

    pub fn into_results_and_va(self) -> (Results, VaArgs) {
        (self.results, self.va_args)
    }

    pub fn push_scope(&mut self, descriptor: ScopeDescriptor) {
        self.referenced.push(self.local.clone());
        self.local = Scope::new(descriptor.size);
    }

    pub fn pop_scope(&mut self) {
        self.local = self
            .referenced
            .pop()
            .expect("Pop should always come after push");
    }

    pub fn load_va(&self, index: usize) -> Value {
        self.va_args.get(index).cloned().unwrap_or(Value::Nil)
    }

    pub fn iter_va(&self) -> impl ExactSizeIterator<Item = &Value> + '_ {
        self.va_args.iter()
    }

    pub fn add_result(&mut self, val: Value) {
        self.results.push(val);
    }

    pub fn extend_results(&mut self, other: impl IntoIterator<Item = Value>) {
        self.results.extend(other.into_iter());
    }

    // TODO(perf): This shouldn't be cloning its values.
    #[track_caller]
    pub fn load(&self, addr: MappedRegister<Register>) -> Value {
        let MappedRegister(Register { scope, offset }) = addr;

        if usize::from(scope) == self.referenced.len() {
            self.local.registers[usize::from(offset)].borrow().clone()
        } else {
            self.referenced[usize::from(scope)].registers[usize::from(offset)]
                .borrow()
                .clone()
        }
    }

    #[track_caller]
    pub fn store(&mut self, addr: MappedRegister<Register>, value: Value) {
        let MappedRegister(Register { scope, offset }) = addr;

        if usize::from(scope) == self.referenced.len() {
            self.local.registers[usize::from(offset)].replace(value);
        } else {
            self.referenced[usize::from(scope)].registers[usize::from(offset)].replace(value);
        }
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

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Function {}

impl std::hash::Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
