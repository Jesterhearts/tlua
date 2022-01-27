use std::{
    collections::{
        btree_map,
        hash_map,
        BTreeMap,
        HashMap,
        HashSet,
    },
    num::NonZeroUsize,
};

use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    OpError,
};
use tlua_parser::ast::identifiers::Ident;

use crate::{
    compiler::{
        unasm::{
            OffsetRegister,
            UnasmFunction,
            UnasmOp,
        },
        LabelId,
        UninitRegister,
    },
    CompileError,
};

pub(super) const GLOBAL_SCOPE: u16 = 0;

/// Manages tracking the maping from identifier to register for a particular
/// scope.
#[derive(Debug, Default)]
pub(super) struct RootScope {
    /// Globals - tracked separately since the runtime may need to perform
    /// initialization or provide access to client code.
    globals: HashMap<Ident, OffsetRegister>,

    /// All locals currently in scope, with the last value for each vec
    /// representing the current register mapped for the local.
    all_locals: HashMap<Ident, Vec<OffsetRegister>>,

    /// The id of the current most recently created scope.
    current_scope_id: usize,
}

impl RootScope {
    pub(super) fn main_function(&mut self) -> FunctionScope {
        let scope_id = self.next_scope_id();
        FunctionScope {
            root: self,
            scope_id,
            scope_depth: NonZeroUsize::new(usize::from(GLOBAL_SCOPE + 1)).unwrap(),
            labels: Default::default(),
            unresolved_jumps: Default::default(),
            next_loop_id: 0,
            next_if_id: 0,
            function: Default::default(),
        }
    }

    pub(super) fn into_globals(self) -> HashMap<Ident, OffsetRegister> {
        self.globals
    }

    fn next_scope_id(&mut self) -> usize {
        self.current_scope_id += 1;
        self.current_scope_id
    }
}

#[derive(Debug)]
pub(super) struct FunctionScope<'function> {
    root: &'function mut RootScope,

    scope_id: usize,
    scope_depth: NonZeroUsize,

    /// The current labels visible in the scope. See [`unresolved_jumps`] for
    /// information on how these are handled.
    labels: HashMap<LabelId, usize>,

    /// A map from label to scope id to a list of unresolved jumps in that
    /// scope.
    ///
    /// Lua's scoping rules are as follows:
    /// - Functions do not allow gotos out of their scopes.
    /// - If a label is declared anywhere in a scope or its parents, it is
    ///   visible to gotos in that scope or any child scope.
    /// - If a local declaration happens, attempting to jump forwards across the
    ///   local declaration is an error.
    ///
    /// We handle these rules using the structure below, which maps label ->
    /// [mapping of scope id -> list of pending jumps].
    ///
    /// Every time we create a new child scope, we give it a new, unique,
    /// monotonically increasing scope id. This means that all children of a
    /// scope will have an ID greater than its own.
    /// This means that we can do a range query for every scope >= this scope on
    /// the btree to get all unresolved jumps for all child scopes and
    /// iterate over them to resolve them.
    /// This solves jumps from children into parent scopes.
    ///
    /// In order to handle locals invaliding jumps, we track two values inside
    /// [`BlockScope`] - `original_scope_id` and `current_scope_id`. Every time
    /// we declare a new local, we set `current_scope_id` to a new unique scope
    /// id. All child scopes after the local declaration will have greater scope
    /// ids than this new one, preserving our ability to do range queries for
    /// valid jump targets.
    /// This allows us to query for jump targets in the range
    /// [`original_scope_id`, `current_scope_id`) to locate pending jumps for a
    /// label that would be jumping into a local variable's scope when a label
    /// is added.
    /// Because we only update the id for the scope in which the local is
    /// declared, we preserve the ability for parent scopes to declare
    /// resolvable labels even when locals are created in the child scope.
    /// This allows this code to compile:
    /// ```lua
    /// if a then
    ///    goto b
    ///    local c
    ///    -- ::b:: would be invalid here.
    /// end
    /// ::b:: -- but it's valid here, since c's scope has ended.
    /// ```
    unresolved_jumps: HashMap<LabelId, BTreeMap<usize, Vec<usize>>>,

    next_loop_id: usize,
    next_if_id: usize,

    function: UnasmFunction,
}

impl<'function> FunctionScope<'function> {
    pub(super) fn start<'block>(&'block mut self) -> BlockScope<'block, 'function> {
        let scope_depth = self.scope_depth;
        let scope_id = self.scope_id;

        BlockScope {
            function_scope: self,
            original_scope_id: scope_id,
            current_scope_id: scope_id,
            scope_depth,
            declared_locals: Default::default(),
            declared_labels: Default::default(),
        }
    }

    pub(super) fn complete(self) -> UnasmFunction {
        self.function
    }

    fn next_if_id(&mut self) -> LabelId {
        let id = self.next_if_id;
        self.next_if_id += 1;

        LabelId::If {
            scope: self.scope_depth.get(),
            id,
        }
    }

    fn push_loop_id(&mut self) -> LabelId {
        let id = self.next_loop_id;
        self.next_loop_id += 1;

        LabelId::Loop {
            scope: self.scope_depth.get(),
            id,
        }
    }

    fn pop_loop_id(&mut self) {
        self.next_loop_id -= 1;
    }

    fn current_loop_id(&self) -> Option<LabelId> {
        self.next_loop_id.checked_sub(1).map(|id| LabelId::Loop {
            scope: self.scope_depth.get(),
            id,
        })
    }
}

#[derive(Debug)]
pub(super) struct BlockScope<'block, 'function> {
    function_scope: &'block mut FunctionScope<'function>,

    original_scope_id: usize,
    current_scope_id: usize,
    scope_depth: NonZeroUsize,

    declared_locals: HashSet<Ident>,
    declared_labels: HashSet<LabelId>,
}

impl Drop for BlockScope<'_, '_> {
    fn drop(&mut self) {
        for decl in self.declared_locals.drain() {
            match self.function_scope.root.all_locals.entry(decl) {
                hash_map::Entry::Occupied(mut shadows) => {
                    let popped = shadows.get_mut().pop();
                    debug_assert!(popped.is_some());

                    if shadows.get_mut().is_empty() {
                        shadows.remove();
                    }
                }
                hash_map::Entry::Vacant(_) => unreachable!("Local decl not in root list."),
            }
        }

        for label in self.declared_labels.drain() {
            let removed = self.function_scope.labels.remove(&label);
            debug_assert!(removed.is_some());
        }
    }
}

impl<'function> BlockScope<'_, 'function> {
    pub(super) fn subscope<'sub>(&'sub mut self) -> BlockScope<'sub, 'function> {
        let scope_id = self.function_scope.root.next_scope_id();
        BlockScope {
            function_scope: self.function_scope,
            original_scope_id: scope_id,
            current_scope_id: scope_id,
            scope_depth: NonZeroUsize::new(self.scope_depth.get() + 1).unwrap(),
            declared_locals: Default::default(),
            declared_labels: Default::default(),
        }
    }

    pub(super) fn new_function(&mut self, params: usize) -> FunctionScope {
        let scope_id = self.function_scope.root.next_scope_id();

        FunctionScope {
            root: self.function_scope.root,
            scope_id,
            scope_depth: NonZeroUsize::new(self.scope_depth.get() + 1).unwrap(),
            labels: Default::default(),
            unresolved_jumps: Default::default(),
            next_loop_id: 0,
            next_if_id: 0,
            function: UnasmFunction {
                named_args: params,
                ..Default::default()
            },
        }
    }

    pub(super) fn next_if_id(&mut self) -> LabelId {
        self.function_scope.next_if_id()
    }

    pub(super) fn push_loop_id(&mut self) -> LabelId {
        self.function_scope.push_loop_id()
    }

    pub(super) fn pop_loop_id(&mut self) {
        self.function_scope.pop_loop_id()
    }

    pub(super) fn current_loop_id(&self) -> Option<LabelId> {
        self.function_scope.current_loop_id()
    }

    pub(super) fn instructions(&self) -> &Vec<UnasmOp> {
        &self.function_scope.function.instructions
    }

    pub(super) fn add_label(&mut self, label: LabelId) -> Result<(), CompileError> {
        let location = self.instructions().len();

        if self.function_scope.labels.insert(label, location).is_some() {
            return Err(CompileError::DuplicateLabel {
                label: format!("{:?}", label),
            });
        }

        let is_new_label = self.declared_labels.insert(label);
        debug_assert!(is_new_label);

        if let Some(maybe_resolve) = self.function_scope.unresolved_jumps.get_mut(&label) {
            if maybe_resolve
                .range(self.original_scope_id..self.current_scope_id)
                .next()
                .is_some()
            {
                return Err(CompileError::JumpIntoLocalScope {
                    label: format!("{:?}", label),
                });
            }

            for pending_jump in maybe_resolve
                .range_mut(self.current_scope_id..)
                .flat_map(|(_, items)| items.drain(..))
            {
                self.function_scope.function.instructions[pending_jump] =
                    opcodes::Jump::from(location).into();
            }
        }

        Ok(())
    }

    pub(super) fn emit_jump_label(&mut self, label: LabelId) -> usize {
        match self.function_scope.labels.get(&label) {
            Some(&location) => self.emit(opcodes::Jump::from(location)),
            None => {
                let position = self.emit(opcodes::Raise::from(OpError::MissingLabel));
                match self.function_scope.unresolved_jumps.entry(label) {
                    hash_map::Entry::Vacant(new_scope_entries) => {
                        new_scope_entries
                            .insert(BTreeMap::from([(self.current_scope_id, vec![position])]));
                    }
                    hash_map::Entry::Occupied(mut scope_entries) => {
                        match scope_entries.get_mut().entry(self.current_scope_id) {
                            btree_map::Entry::Vacant(new_list) => {
                                new_list.insert(vec![position]);
                            }
                            btree_map::Entry::Occupied(mut list) => {
                                list.get_mut().push(position);
                            }
                        }
                    }
                };

                position
            }
        }
    }

    pub(super) fn emit(&mut self, opcode: impl Into<UnasmOp>) -> usize {
        let position = self.instructions().len();
        self.function_scope
            .function
            .instructions
            .push(opcode.into());
        position
    }

    pub(super) fn overwrite(&mut self, location: usize, opcode: impl Into<UnasmOp>) {
        self.function_scope.function.instructions[location] = opcode.into();
    }

    pub(super) fn total_locals(&self) -> usize {
        self.declared_locals.len()
    }

    pub(super) fn total_anons(&self) -> usize {
        self.function_scope.function.anon_registers
    }

    pub(super) fn get_in_scope(&mut self, ident: Ident) -> Result<OffsetRegister, CompileError> {
        match self.function_scope.root.all_locals.entry(ident) {
            hash_map::Entry::Occupied(exists) => Ok(exists
                .get()
                .last()
                .copied()
                .expect("Empty shadows lists should be removed.")),
            hash_map::Entry::Vacant(unknown) => {
                // No ident is in scope, must be a global
                let offset_register =
                    OffsetRegister {
                        source_scope_depth: GLOBAL_SCOPE,
                        offset: self.function_scope.root.globals.len().try_into().map_err(
                            |_| CompileError::TooManyGlobals {
                                max: u16::MAX.into(),
                            },
                        )?,
                    };

                unknown.insert(vec![offset_register]);
                self.function_scope
                    .root
                    .globals
                    .insert(ident, offset_register);

                Ok(offset_register)
            }
        }
    }

    pub(super) fn new_anonymous(&mut self) -> UninitRegister<AnonymousRegister> {
        let reg = self.function_scope.function.anon_registers;
        self.function_scope.function.anon_registers += 1;

        UninitRegister {
            register: reg.into(),
        }
    }

    pub(super) fn new_local(
        &mut self,
        ident: Ident,
    ) -> Result<UninitRegister<OffsetRegister>, CompileError> {
        self.current_scope_id = self.function_scope.root.next_scope_id();

        let offset_register = OffsetRegister {
            source_scope_depth: if self.scope_depth.get() >= usize::from(u16::MAX) {
                return Err(CompileError::ScopeNestingTooDeep {
                    max: usize::from(u16::MAX - 1),
                });
            } else {
                self.scope_depth.get().try_into().unwrap()
            },
            offset: self
                .function_scope
                .function
                .local_registers
                .try_into()
                .map_err(|_| CompileError::TooManyLocals {
                    max: u16::MAX.into(),
                })?,
        };
        self.function_scope.function.local_registers += 1;

        if self.declared_locals.contains(&ident) {
            *self
                .function_scope
                .root
                .all_locals
                .get_mut(&ident)
                .and_then(|vec| vec.last_mut())
                .expect("Previous local decl") = offset_register;
        } else {
            self.declared_locals.insert(ident);

            match self.function_scope.root.all_locals.entry(ident) {
                hash_map::Entry::Occupied(mut shadow_list) => {
                    shadow_list.get_mut().push(offset_register);
                }
                hash_map::Entry::Vacant(first_decl) => {
                    first_decl.insert(vec![offset_register]);
                }
            }
        }

        Ok(offset_register.into())
    }
}
