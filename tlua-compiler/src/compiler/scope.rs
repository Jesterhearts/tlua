use std::{
    collections::{
        hash_map::Entry,
        HashMap,
        HashSet,
    },
    num::NonZeroUsize,
};

use tlua_bytecode::AnonymousRegister;
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
}

impl RootScope {
    pub(super) fn main_function(&mut self) -> FunctionScope {
        FunctionScope {
            root: self,
            scope_id: NonZeroUsize::new(usize::from(GLOBAL_SCOPE + 1)).unwrap(),
            function: Default::default(),
        }
    }

    pub(super) fn into_globals(self) -> HashMap<Ident, OffsetRegister> {
        self.globals
    }
}

#[derive(Debug)]
pub(super) struct FunctionScope<'function> {
    root: &'function mut RootScope,

    scope_id: NonZeroUsize,

    function: UnasmFunction,
}

impl<'function> FunctionScope<'function> {
    pub(super) fn start<'block>(&'block mut self) -> BlockScope<'block, 'function> {
        let scope_id = self.scope_id;
        BlockScope {
            function_scope: self,
            scope_id,
            declared_locals: Default::default(),
            declared_labels: Default::default(),
            unresolved_jumps: Default::default(),
        }
    }

    pub(super) fn complete(self) -> UnasmFunction {
        self.function
    }
}

#[derive(Debug)]
pub(super) struct BlockScope<'block, 'function> {
    function_scope: &'block mut FunctionScope<'function>,

    scope_id: NonZeroUsize,

    declared_locals: HashSet<Ident>,
    declared_labels: HashSet<LabelId>,

    unresolved_jumps: HashMap<LabelId, usize>,
}

impl Drop for BlockScope<'_, '_> {
    fn drop(&mut self) {
        for decl in self.declared_locals.drain() {
            match self.function_scope.root.all_locals.entry(decl) {
                Entry::Occupied(mut shadows) => {
                    let popped = shadows.get_mut().pop();
                    debug_assert!(popped.is_some());

                    if shadows.get_mut().is_empty() {
                        shadows.remove();
                    }
                }
                Entry::Vacant(_) => unreachable!("Local decl not in root list."),
            }
        }
    }
}

impl<'function> BlockScope<'_, 'function> {
    pub(super) fn subscope<'sub>(&'sub mut self) -> BlockScope<'sub, 'function> {
        BlockScope {
            function_scope: self.function_scope,
            scope_id: NonZeroUsize::new(self.scope_id.get() + 1).unwrap(),
            declared_locals: Default::default(),
            declared_labels: Default::default(),
            unresolved_jumps: Default::default(),
        }
    }

    pub(super) fn new_function(&mut self, params: usize) -> FunctionScope {
        FunctionScope {
            root: self.function_scope.root,
            scope_id: NonZeroUsize::new(self.scope_id.get() + 1).unwrap(),
            function: UnasmFunction {
                named_args: params,
                ..Default::default()
            },
        }
    }

    pub(super) fn instructions(&self) -> &Vec<UnasmOp> {
        &self.function_scope.function.instructions
    }

    pub(super) fn emit(&mut self, opcode: impl Into<UnasmOp>) {
        self.function_scope
            .function
            .instructions
            .push(opcode.into());
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
            Entry::Occupied(exists) => Ok(exists
                .get()
                .last()
                .copied()
                .expect("Empty shadows lists should be removed.")),
            Entry::Vacant(unknown) => {
                // No ident is in scope, must be a global
                let offset_register =
                    OffsetRegister {
                        source_scope: GLOBAL_SCOPE,
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
        let offset_register = OffsetRegister {
            source_scope: if self.scope_id.get() >= usize::from(u16::MAX) {
                return Err(CompileError::ScopeNestingTooDeep {
                    max: usize::from(u16::MAX - 1),
                });
            } else {
                self.scope_id.get().try_into().unwrap()
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
                Entry::Occupied(mut shadow_list) => {
                    shadow_list.get_mut().push(offset_register);
                }
                Entry::Vacant(first_decl) => {
                    first_decl.insert(vec![offset_register]);
                }
            }
        }

        Ok(offset_register.into())
    }
}
