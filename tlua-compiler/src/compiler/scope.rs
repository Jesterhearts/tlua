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
        unasm::OffsetRegister,
        UninitRegister,
    },
    CompileError,
};

pub(super) const GLOBAL_SCOPE: u16 = 0;

/// Manages tracking the maping from identifier to register for a particular
/// scope.
#[derive(Debug, Default)]
pub(super) struct Scope {
    /// Globals - tracked separately since the runtime may need to perform
    /// initialization or provide access to client code.
    pub(super) globals: HashMap<Ident, OffsetRegister>,

    /// All locals currently in scope, with the last value for each vec
    /// representing the current register mapped for the local.
    in_scope: HashMap<Ident, Vec<OffsetRegister>>,
}

impl Scope {
    pub(super) fn new_context(&mut self, parent_id: usize) -> ScopeContext {
        let scope_id = NonZeroUsize::new(parent_id + 1).unwrap();

        ScopeContext {
            scope: self,
            scope_id,
            total_locals: 0,
            total_anons: 0,
            local_decls: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(super) struct ScopeContext<'function> {
    scope: &'function mut Scope,

    local_decls: HashSet<Ident>,

    pub(super) scope_id: NonZeroUsize,
    pub(super) total_locals: usize,
    pub(super) total_anons: usize,
}

impl Drop for ScopeContext<'_> {
    fn drop(&mut self) {
        // Cleanup shadows
        for decl in self.local_decls.drain() {
            let popped = self.scope.in_scope.get_mut(&decl).and_then(|vec| vec.pop());
            debug_assert!(popped.is_some());

            if self.scope.in_scope[&decl].is_empty() {
                self.scope.in_scope.remove(&decl);
            }
        }
    }
}

impl ScopeContext<'_> {
    pub(super) fn subcontext(&mut self) -> ScopeContext {
        self.scope.new_context(self.scope_id.get())
    }

    pub(super) fn get_in_scope(&mut self, ident: Ident) -> Result<OffsetRegister, CompileError> {
        match self.scope.in_scope.entry(ident) {
            Entry::Occupied(exists) => Ok(exists
                .get()
                .last()
                .copied()
                .expect("Empty shadows lists should be removed.")),
            Entry::Vacant(unknown) => {
                // No ident is in scope, must be a global
                let offset_register = OffsetRegister {
                    source_scope: GLOBAL_SCOPE,
                    offset: self.scope.globals.len().try_into().map_err(|_| {
                        CompileError::TooManyGlobals {
                            max: u16::MAX.into(),
                        }
                    })?,
                };

                unknown.insert(vec![offset_register]);
                self.scope.globals.insert(ident, offset_register);

                Ok(offset_register)
            }
        }
    }

    pub(super) fn new_anonymous(&mut self) -> UninitRegister<AnonymousRegister> {
        let reg = self.total_anons;
        self.total_anons += 1;

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
                .total_locals
                .try_into()
                .map_err(|_| CompileError::TooManyLocals {
                    max: u16::MAX.into(),
                })?,
        };
        self.total_locals += 1;

        if self.local_decls.contains(&ident) {
            *self
                .scope
                .in_scope
                .get_mut(&ident)
                .and_then(|vec| vec.last_mut())
                .expect("Previous local decl") = offset_register;
        } else {
            self.local_decls.insert(ident);

            match self.scope.in_scope.entry(ident) {
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
