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
    ByteCodeError,
    OpError,
};
use tlua_parser::ast::identifiers::Ident;

use crate::{
    compiler::{
        unasm::{
            MappedLocalRegister,
            OffsetRegister,
            UnasmFunction,
            UnasmOp,
        },
        HasVaArgs,
        InitRegister,
        LabelId,
        UninitRegister,
    },
    Chunk,
    CompileError,
    FuncId,
    NodeOutput,
};

const GLOBAL_SCOPE: u16 = 0;

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

    /// The current list of functions created in this scope.
    functions: Vec<UnasmFunction>,
}

impl RootScope {
    pub(super) fn start_main(&mut self) -> FunctionScope {
        let scope_id = self.next_scope_id();
        let scope_depth = NonZeroUsize::new(usize::from(GLOBAL_SCOPE + 1)).unwrap();
        FunctionScope::new(self, scope_id, scope_depth, HasVaArgs::None, 0)
    }

    pub(super) fn into_chunk(self, main: UnasmFunction) -> Chunk {
        Chunk {
            globals_map: self
                .globals
                .into_iter()
                .map(|(global, reg)| {
                    debug_assert_eq!(reg.source_scope_depth, GLOBAL_SCOPE);
                    (global, reg.offset.into())
                })
                .collect(),
            functions: self
                .functions
                .into_iter()
                .map(|func| func.into_function())
                .collect(),
            main: main.into_function(),
        }
    }

    fn next_scope_id(&mut self) -> usize {
        self.current_scope_id += 1;
        self.current_scope_id
    }
}

#[derive(Debug)]
pub(crate) struct FunctionScope<'function> {
    root_scope: &'function mut RootScope,

    scope_id: usize,
    scope_depth: NonZeroUsize,

    has_va_args: HasVaArgs,

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
    fn new(
        root_scope: &'function mut RootScope,
        scope_id: usize,
        scope_depth: NonZeroUsize,
        has_va_args: HasVaArgs,
        argc: usize,
    ) -> Self {
        Self {
            root_scope,
            scope_id,
            scope_depth,
            has_va_args,
            labels: Default::default(),
            unresolved_jumps: Default::default(),
            next_loop_id: 0,
            next_if_id: 0,
            function: UnasmFunction {
                named_args: argc,
                ..Default::default()
            },
        }
    }

    pub(crate) fn start<'block>(&'block mut self) -> BlockScope<'block, 'function> {
        let scope_id = self.scope_id;
        let scope_depth = self.scope_depth;

        BlockScope::new(self, scope_id, scope_depth, None)
    }

    pub(crate) fn complete(self) -> FuncId {
        let id = self.root_scope.functions.len();
        self.root_scope.functions.push(self.function);
        FuncId::from(id)
    }

    pub(super) fn complete_main(self) -> UnasmFunction {
        self.function
    }

    fn next_if_id(&mut self) -> LabelId {
        let id = self.next_if_id;
        self.next_if_id += 1;

        LabelId::If { id }
    }

    fn push_loop_id(&mut self) -> LabelId {
        let id = self.next_loop_id;
        self.next_loop_id += 1;

        LabelId::Loop { id }
    }

    fn pop_loop_id(&mut self) {
        self.next_loop_id -= 1;
    }

    fn current_loop_id(&self) -> Option<LabelId> {
        self.next_loop_id
            .checked_sub(1)
            .map(|id| LabelId::Loop { id })
    }
}

#[derive(Debug)]
pub(crate) struct BlockScope<'block, 'function> {
    function_scope: &'block mut FunctionScope<'function>,

    original_scope_id: usize,
    current_scope_id: usize,
    scope_depth: NonZeroUsize,

    pending_scope_push: Option<usize>,

    declared_locals: HashSet<Ident>,
    declared_labels: HashSet<LabelId>,
}

impl Drop for BlockScope<'_, '_> {
    fn drop(&mut self) {
        if let Some(location) = self.pending_scope_push {
            self.overwrite(
                location,
                opcodes::ScopeDescriptor::from(self.declared_locals.len()),
            );
            self.emit(opcodes::Op::PopScope);
        }

        for decl in self.declared_locals.drain() {
            match self.function_scope.root_scope.all_locals.entry(decl) {
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

impl<'block, 'function> BlockScope<'block, 'function> {
    pub(crate) fn enter<'scope>(&'scope mut self) -> Scope<'scope, 'block, 'function> {
        Scope { block_scope: self }
    }

    fn new_with_pushed_scope(
        function_scope: &'block mut FunctionScope<'function>,
        scope_id: usize,
        scope_depth: NonZeroUsize,
    ) -> Self {
        let pending_scope_push = function_scope.function.instructions.len();
        function_scope.function.instructions.push(
            opcodes::Raise::from(OpError::ByteCodeError {
                err: ByteCodeError::MissingScopeDescriptor,
                offset: pending_scope_push,
            })
            .into(),
        );

        Self::new(
            function_scope,
            scope_id,
            scope_depth,
            Some(pending_scope_push),
        )
    }

    fn new(
        function_scope: &'block mut FunctionScope<'function>,
        scope_id: usize,
        scope_depth: NonZeroUsize,
        pending_scope_push: Option<usize>,
    ) -> Self {
        BlockScope {
            function_scope,
            original_scope_id: scope_id,
            current_scope_id: scope_id,
            scope_depth,
            pending_scope_push,
            declared_locals: Default::default(),
            declared_labels: Default::default(),
        }
    }

    fn overwrite(&mut self, location: usize, opcode: impl Into<UnasmOp>) {
        self.function_scope.function.instructions[location] = opcode.into();
    }

    fn emit(&mut self, opcode: impl Into<UnasmOp>) -> usize {
        let position = self.function_scope.function.instructions.len();
        self.function_scope
            .function
            .instructions
            .push(opcode.into());

        position
    }
}

#[derive(Debug)]
pub(crate) struct Scope<'context, 'block, 'function> {
    block_scope: &'context mut BlockScope<'block, 'function>,
}

impl<'context, 'block, 'function> Scope<'context, 'block, 'function> {
    /// Check if varargs are available in scope
    pub(crate) fn check_varargs(&self) -> Result<(), CompileError> {
        match self.block_scope.function_scope.has_va_args {
            HasVaArgs::None => Err(CompileError::NoVarArgsAvailable),
            HasVaArgs::Some => Ok(()),
        }
    }

    /// Get the current offset in the instruction stream.
    pub(crate) fn next_instruction(&self) -> usize {
        self.block_scope.function_scope.function.instructions.len()
    }

    /// Add a label tracking the current instruction position that can be
    /// referenced by labeled jumps.
    pub(crate) fn label_current_instruction(&mut self, label: LabelId) -> Result<(), CompileError> {
        let label = label;
        let location = self.block_scope.function_scope.function.instructions.len();

        if self
            .block_scope
            .function_scope
            .labels
            .insert(label, location)
            .is_some()
        {
            return Err(CompileError::DuplicateLabel {
                label: format!("{:?}", label),
            });
        }

        let is_new_label = self.block_scope.declared_labels.insert(label);
        debug_assert!(is_new_label);

        if let Some(maybe_resolve) = self
            .block_scope
            .function_scope
            .unresolved_jumps
            .get_mut(&label)
        {
            if maybe_resolve
                .range(self.block_scope.original_scope_id..self.block_scope.current_scope_id)
                .next()
                .is_some()
            {
                return Err(CompileError::JumpIntoLocalScope {
                    label: format!("{:?}", label),
                });
            }

            for pending_jump in maybe_resolve
                .range_mut(self.block_scope.current_scope_id..)
                .flat_map(|(_, items)| items.drain(..))
            {
                self.block_scope.function_scope.function.instructions[pending_jump] =
                    opcodes::Jump::from(location).into();
            }
        }

        Ok(())
    }

    /// Create a new, unique label for an if statement.
    pub(crate) fn create_if_label(&mut self) -> LabelId {
        self.block_scope.function_scope.next_if_id()
    }

    /// Get the current active loop label if the current scope is nested inside
    /// of a loop.
    pub(crate) fn current_loop_label(&self) -> Option<LabelId> {
        self.block_scope.function_scope.current_loop_id()
    }

    /// Create a new loop label. The caller must call pop_loop_label after
    /// using.
    pub(crate) fn push_loop_label(&mut self) -> LabelId {
        self.block_scope.function_scope.push_loop_id()
    }

    /// Pop the current loop label.
    pub(crate) fn pop_loop_label(&mut self) {
        self.block_scope.function_scope.pop_loop_id()
    }

    /// Emit an instruction jumping to a label. If the specified label does not
    /// exist, it will default to raising an error. If the label is added later
    /// in the scope, the instruction will be updated to jump to that location.
    pub(crate) fn emit_jump_label(&mut self, label: LabelId) -> usize {
        match self.block_scope.function_scope.labels.get(&label) {
            Some(&location) => self.emit(opcodes::Jump::from(location)),
            None => {
                let position = self.emit(opcodes::Raise::from(OpError::MissingLabel));
                match self
                    .block_scope
                    .function_scope
                    .unresolved_jumps
                    .entry(label)
                {
                    hash_map::Entry::Vacant(new_scope_entries) => {
                        new_scope_entries.insert(BTreeMap::from([(
                            self.block_scope.current_scope_id,
                            vec![position],
                        )]));
                    }
                    hash_map::Entry::Occupied(mut scope_entries) => {
                        match scope_entries
                            .get_mut()
                            .entry(self.block_scope.current_scope_id)
                        {
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

    /// Emit a new opcode to the current instruction stream. Returns the
    /// location in the instruction stream.
    pub(crate) fn emit(&mut self, opcode: impl Into<UnasmOp>) -> usize {
        self.block_scope.emit(opcode)
    }

    /// Overwrite the instruction at location.
    pub(crate) fn overwrite(&mut self, location: usize, opcode: impl Into<UnasmOp>) {
        self.block_scope.overwrite(location, opcode)
    }

    /// Map a new register for a local variable.
    pub(crate) fn new_local(
        &mut self,
        ident: Ident,
    ) -> Result<UninitRegister<MappedLocalRegister>, CompileError> {
        {
            self.block_scope.current_scope_id =
                self.block_scope.function_scope.root_scope.next_scope_id();

            let offset_register = OffsetRegister {
                source_scope_depth: if self.block_scope.scope_depth.get() >= usize::from(u16::MAX) {
                    return Err(CompileError::ScopeNestingTooDeep {
                        max: usize::from(u16::MAX - 1),
                    });
                } else {
                    self.block_scope.scope_depth.get().try_into().unwrap()
                },
                offset: self
                    .block_scope
                    .declared_locals
                    .len()
                    .try_into()
                    .map_err(|_| CompileError::TooManyLocals {
                        max: u16::MAX.into(),
                    })?,
            };
            self.block_scope.function_scope.function.local_registers += 1;

            if self.block_scope.declared_locals.contains(&ident) {
                *self
                    .block_scope
                    .function_scope
                    .root_scope
                    .all_locals
                    .get_mut(&ident)
                    .and_then(|vec| vec.last_mut())
                    .expect("Previous local decl") = offset_register;
            } else {
                self.block_scope.declared_locals.insert(ident);

                match self
                    .block_scope
                    .function_scope
                    .root_scope
                    .all_locals
                    .entry(ident)
                {
                    hash_map::Entry::Occupied(mut shadow_list) => {
                        shadow_list.get_mut().push(offset_register);
                    }
                    hash_map::Entry::Vacant(first_decl) => {
                        first_decl.insert(vec![offset_register]);
                    }
                }
            }

            Ok(MappedLocalRegister::from(offset_register).into())
        }
    }

    /// Allocate a new anonymous register.
    pub(crate) fn new_anon_reg(&mut self) -> UninitRegister<AnonymousRegister> {
        let reg = self.block_scope.function_scope.function.anon_registers;
        self.block_scope.function_scope.function.anon_registers += 1;

        UninitRegister {
            register: reg.into(),
        }
    }

    /// Allocate a sequence of anonymous registers.
    pub(crate) fn new_anon_reg_range(
        &mut self,
        size: usize,
    ) -> impl ExactSizeIterator<Item = UninitRegister<AnonymousRegister>> + Clone {
        let start = { self.block_scope.function_scope.function.anon_registers };
        let range = start..(start + size);
        for _ in range.clone() {
            let _ = self.new_anon_reg().no_init_needed();
        }

        range.map(|idx| UninitRegister::from(AnonymousRegister::from(idx)))
    }

    /// Allocate a new anonymous register.
    pub(crate) fn output_to_reg_reuse_anon(&mut self, output: NodeOutput) -> AnonymousRegister {
        match output {
            NodeOutput::Immediate(imm) => imm,
            other => self.new_anon_reg().init_from_node_output(self, other),
        }
    }

    /// Lookup the appropriate register for a specific identifier.
    pub(crate) fn read_variable(
        &mut self,
        ident: Ident,
    ) -> Result<MappedLocalRegister, CompileError> {
        match self
            .block_scope
            .function_scope
            .root_scope
            .all_locals
            .entry(ident)
        {
            hash_map::Entry::Occupied(exists) => Ok(exists
                .get()
                .last()
                .copied()
                .expect("Empty shadows lists should be removed.")
                .into()),
            hash_map::Entry::Vacant(unknown) => {
                // No ident is in scope, must be a global
                let offset_register = OffsetRegister {
                    source_scope_depth: GLOBAL_SCOPE,
                    offset: self
                        .block_scope
                        .function_scope
                        .root_scope
                        .globals
                        .len()
                        .try_into()
                        .map_err(|_| CompileError::TooManyGlobals {
                            max: u16::MAX.into(),
                        })?,
                };

                unknown.insert(vec![offset_register]);
                self.block_scope
                    .function_scope
                    .root_scope
                    .globals
                    .insert(ident, offset_register);

                Ok(offset_register.into())
            }
        }
    }

    /// Instruct the compiler to emit a sequence of instruction corresponding to
    /// raising an error with a compile-time known type.
    pub(crate) fn write_raise(&mut self, err: OpError) -> OpError {
        self.emit(opcodes::Raise::from(err));
        err
    }

    pub(crate) fn new_function(&mut self, has_va_args: HasVaArgs, argc: usize) -> FunctionScope {
        let scope_id = self.block_scope.function_scope.root_scope.next_scope_id();
        let scope_depth = NonZeroUsize::new(self.block_scope.scope_depth.get() + 1).unwrap();

        FunctionScope::new(
            self.block_scope.function_scope.root_scope,
            scope_id,
            scope_depth,
            has_va_args,
            argc,
        )
    }

    pub(crate) fn new_block<'sub>(&'sub mut self) -> BlockScope<'sub, 'function> {
        let scope_id = self.block_scope.function_scope.root_scope.next_scope_id();
        let scope_depth = NonZeroUsize::new(self.block_scope.scope_depth.get() + 1).unwrap();

        BlockScope::new_with_pushed_scope(self.block_scope.function_scope, scope_id, scope_depth)
    }
}
