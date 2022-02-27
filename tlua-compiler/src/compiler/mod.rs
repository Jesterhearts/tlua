use std::{
    marker::PhantomData,
    ops::Range,
};

use derive_more::From;
use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    Constant,
    ImmediateRegister,
};
use tlua_parser::{
    block::Block,
    identifiers::Ident,
    StringTable,
};

use crate::{
    block::emit_block,
    BuiltinType,
    Chunk,
    CompileError,
    FuncId,
};

mod scope;
pub(super) mod unasm;

pub(crate) use scope::Scope;

use self::{
    scope::*,
    unasm::*,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum HasVaArgs {
    None,
    Some,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LabelId {
    Named(Ident),
    If { id: usize },
    Loop { id: usize },
}

#[derive(Debug)]
pub(crate) struct Compiler {
    root: RootScope,
}

impl Compiler {
    pub(crate) fn new(strings: StringTable) -> Self {
        Self {
            root: RootScope::new(strings),
        }
    }

    pub(crate) fn compile_ast(mut self, ast: Block) -> Result<Chunk, CompileError> {
        let main = {
            let mut main = self.root.start_main();
            {
                let mut block = main.start();
                let mut scope = block.enter();

                emit_block(&mut scope, &ast)?;
            }

            main.complete_main()
        };

        Ok(self.root.into_chunk(main))
    }
}

#[derive(Debug)]
pub(crate) enum JumpTemplate<Op> {
    Unconditional {
        location: usize,
    },
    Conditional {
        location: usize,
        reg: ImmediateRegister,
        op: PhantomData<Op>,
    },
}

impl<Op: From<(ImmediateRegister, usize)> + Into<UnasmOp>> JumpTemplate<Op> {
    pub(crate) fn unconditional_at(location: usize) -> Self {
        Self::Unconditional { location }
    }

    pub(crate) fn conditional_at(location: usize, reg: ImmediateRegister) -> Self {
        Self::Conditional {
            location,
            reg,
            op: Default::default(),
        }
    }

    pub(crate) fn resolve_to(self, target: usize, scope: &mut Scope) {
        match self {
            JumpTemplate::Unconditional { location } => {
                scope.overwrite(location, opcodes::Jump::from(target))
            }
            JumpTemplate::Conditional {
                location,
                reg,
                op: _,
            } => scope.overwrite(location, Op::from((reg, target))),
        }
    }
}

#[derive(Debug, Clone, From)]
#[must_use]
pub(crate) struct UninitRegister<RegisterTy> {
    register: RegisterTy,
}

#[derive(Debug, Clone, From)]
#[must_use]
pub(crate) struct UninitRegisterRange {
    range: Range<usize>,
}

impl UninitRegisterRange {
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = UninitRegister<ImmediateRegister>> {
        self.range
            .clone()
            .map(ImmediateRegister::from)
            .map(UninitRegister::from)
    }
}

pub(crate) trait InitRegister<RegisterTy = Self>: Sized {
    /// Indicate that the register should always init to nil, and needs no
    /// special handling.
    fn no_init_needed(self) -> RegisterTy;

    /// Indicate that the register should be initialized from a return value.
    fn init_from_ret(self, scope: &mut Scope) -> RegisterTy;

    /// Indicate that the register should be initialized to a constant. If the
    /// constant is always nil, please use init_from_nil.
    fn init_from_const(self, scope: &mut Scope, value: Constant) -> RegisterTy;

    /// Indicate the the register should be initialized by allocating a
    /// function.
    fn init_alloc_fn(self, scope: &mut Scope, value: FuncId) -> RegisterTy;

    /// Indicate the the register shoudl be initialized by allocating a table
    fn init_alloc_table(self, scope: &mut Scope) -> RegisterTy;

    /// Indicate that the register should be initialized from another register.
    fn init_from_immediate(self, scope: &mut Scope, other: ImmediateRegister) -> RegisterTy;

    /// Indicate that the register should be initialized from another register.
    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> RegisterTy;

    /// Indicate that the register should be initialized from a table entry
    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> RegisterTy;

    /// Indicate that the register should be initialized from a variadic
    /// argument;
    fn init_from_va(self, scope: &mut Scope, index: usize) -> RegisterTy;
}

impl InitRegister for ImmediateRegister {
    fn no_init_needed(self) -> Self {
        self
    }

    fn init_from_ret(self, scope: &mut Scope) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::ConsumeRetRange::from((usize::from(reg), 1)));
        reg
    }

    fn init_from_const(self, scope: &mut Scope, value: Constant) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::LoadConstant::from((reg, value)));
        reg
    }

    fn init_alloc_fn(self, scope: &mut Scope, value: FuncId) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::Alloc::from((
            reg,
            BuiltinType::Function(value).into(),
        )));

        reg
    }

    fn init_alloc_table(self, scope: &mut Scope) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::Alloc::from((reg, BuiltinType::Table.into())));

        reg
    }

    fn init_from_immediate(self, scope: &mut Scope, other: ImmediateRegister) -> Self {
        let reg = self.no_init_needed();
        if other != reg {
            scope.emit(opcodes::DuplicateRegister::from((reg, other)));
        }
        reg
    }

    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::LoadRegister::from((reg, other)));
        reg
    }

    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::Lookup::from((reg, table, index)));
        reg
    }

    fn init_from_va(self, scope: &mut Scope, index: usize) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::LoadVa::from((usize::from(reg), index, 1)));
        reg
    }
}

impl InitRegister for MappedLocalRegister {
    fn no_init_needed(self) -> Self {
        self
    }

    fn init_from_ret(self, scope: &mut Scope) -> Self {
        let imm = scope.push_immediate().init_from_ret(scope);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
        self.init_from_immediate(&mut scope, imm)
    }

    fn init_from_const(self, scope: &mut Scope, value: Constant) -> Self {
        let imm = scope.push_immediate().init_from_const(scope, value);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
        self.init_from_immediate(&mut scope, imm)
    }

    fn init_alloc_fn(self, scope: &mut Scope, value: FuncId) -> Self {
        let imm = scope.push_immediate().init_alloc_fn(scope, value);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
        self.init_from_immediate(&mut scope, imm)
    }

    fn init_alloc_table(self, scope: &mut Scope) -> Self {
        let imm = scope.push_immediate().init_alloc_table(scope);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
        self.init_from_immediate(&mut scope, imm)
    }

    fn init_from_immediate(self, scope: &mut Scope, other: ImmediateRegister) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::Store::from((reg, other)));
        reg
    }

    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> Self {
        if other != self {
            let imm = scope.push_immediate().init_from_mapped_reg(scope, other);
            let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
            self.init_from_immediate(&mut scope, imm)
        } else {
            self
        }
    }

    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> Self {
        let imm = scope
            .push_immediate()
            .init_from_table_entry(scope, table, index);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
        self.init_from_immediate(&mut scope, imm)
    }

    fn init_from_va(self, scope: &mut Scope, index: usize) -> Self {
        let imm = scope.push_immediate().init_from_va(scope, index);
        let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(imm));
        self.init_from_immediate(&mut scope, imm)
    }
}

impl<RegisterTy> InitRegister<RegisterTy> for UninitRegister<RegisterTy>
where
    RegisterTy: InitRegister,
{
    fn no_init_needed(self) -> RegisterTy {
        self.register
    }

    fn init_from_ret(self, scope: &mut Scope) -> RegisterTy {
        self.register.init_from_ret(scope)
    }

    fn init_from_const(self, scope: &mut Scope, value: Constant) -> RegisterTy {
        self.register.init_from_const(scope, value)
    }

    fn init_alloc_fn(self, scope: &mut Scope, value: FuncId) -> RegisterTy {
        self.register.init_alloc_fn(scope, value)
    }

    fn init_alloc_table(self, scope: &mut Scope) -> RegisterTy {
        self.register.init_alloc_table(scope)
    }

    fn init_from_immediate(self, scope: &mut Scope, other: ImmediateRegister) -> RegisterTy {
        self.register.init_from_immediate(scope, other)
    }

    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> RegisterTy {
        self.register.init_from_mapped_reg(scope, other)
    }

    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> RegisterTy {
        self.register.init_from_table_entry(scope, table, index)
    }

    fn init_from_va(self, scope: &mut Scope, index: usize) -> RegisterTy {
        self.register.init_from_va(scope, index)
    }
}
