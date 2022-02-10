use derive_more::From;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
};
use tlua_parser::ast::{
    block::Block,
    identifiers::Ident,
};

use crate::{
    block::emit_block,
    constant::Constant,
    BuiltinType,
    Chunk,
    CompileError,
    FuncId,
    NodeOutput,
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

#[derive(Debug, Default)]
pub(crate) struct Compiler {
    root: RootScope,
}

impl Compiler {
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

pub(crate) trait InitRegister<RegisterTy = Self>: Sized {
    /// Indicate that the register should always init to nil, and needs no
    /// special handling.
    fn no_init_needed(self) -> RegisterTy;

    /// Initialize the register from node output.
    fn init_from_node_output(self, scope: &mut Scope, value: NodeOutput) -> RegisterTy {
        match value {
            NodeOutput::Constant(value) => self.init_from_const(scope, value),
            NodeOutput::Immediate(source) => self.init_from_anon_reg(scope, source),
            NodeOutput::MappedRegister(source) => self.init_from_mapped_reg(scope, source),
            NodeOutput::TableEntry { table, index } => {
                self.init_from_table_entry(scope, table, index)
            }
            NodeOutput::ReturnValues => self.init_from_ret(scope),
            NodeOutput::VAStack => self.init_from_va(scope, 0),
            NodeOutput::Err(_) => self.no_init_needed(),
        }
    }

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
    fn init_from_anon_reg(self, scope: &mut Scope, other: AnonymousRegister) -> RegisterTy;

    /// Indicate that the register should be initialized from another register.
    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> RegisterTy;

    /// Indicate that the register should be initialized from a table entry
    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> RegisterTy;

    /// Indicate that the register should be initialized from a variadic
    /// argument;
    fn init_from_va(self, scope: &mut Scope, index: usize) -> RegisterTy;
}

impl InitRegister for AnonymousRegister {
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
        scope.emit(opcodes::LoadConstant::from((reg, value.into())));
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

    fn init_from_anon_reg(self, scope: &mut Scope, other: AnonymousRegister) -> Self {
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
        table: AnonymousRegister,
        index: AnonymousRegister,
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
        let anon = scope.new_anon_reg().init_from_ret(scope);
        self.init_from_anon_reg(scope, anon)
    }

    fn init_from_const(self, scope: &mut Scope, value: Constant) -> Self {
        let anon = scope.new_anon_reg().init_from_const(scope, value);
        self.init_from_anon_reg(scope, anon)
    }

    fn init_alloc_fn(self, scope: &mut Scope, value: FuncId) -> Self {
        let anon = scope.new_anon_reg().init_alloc_fn(scope, value);
        self.init_from_anon_reg(scope, anon)
    }

    fn init_alloc_table(self, scope: &mut Scope) -> Self {
        let anon = scope.new_anon_reg().init_alloc_table(scope);
        self.init_from_anon_reg(scope, anon)
    }

    fn init_from_anon_reg(self, scope: &mut Scope, other: AnonymousRegister) -> Self {
        let reg = self.no_init_needed();
        scope.emit(opcodes::Store::from((reg, other)));
        reg
    }

    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> Self {
        let anon = scope.new_anon_reg().init_from_mapped_reg(scope, other);
        self.init_from_anon_reg(scope, anon)
    }

    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> Self {
        let anon = scope
            .new_anon_reg()
            .init_from_table_entry(scope, table, index);
        self.init_from_anon_reg(scope, anon)
    }

    fn init_from_va(self, scope: &mut Scope, index: usize) -> Self {
        let anon = scope.new_anon_reg().init_from_va(scope, index);
        self.init_from_anon_reg(scope, anon)
    }
}

#[derive(Debug, Clone, From)]
#[must_use]
pub(crate) struct UninitRegister<RegisterTy> {
    register: RegisterTy,
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

    fn init_from_anon_reg(self, scope: &mut Scope, other: AnonymousRegister) -> RegisterTy {
        self.register.init_from_anon_reg(scope, other)
    }

    fn init_from_mapped_reg(self, scope: &mut Scope, other: MappedLocalRegister) -> RegisterTy {
        self.register.init_from_mapped_reg(scope, other)
    }

    fn init_from_table_entry(
        self,
        scope: &mut Scope,
        table: AnonymousRegister,
        index: AnonymousRegister,
    ) -> RegisterTy {
        self.register.init_from_table_entry(scope, table, index)
    }

    fn init_from_va(self, scope: &mut Scope, index: usize) -> RegisterTy {
        self.register.init_from_va(scope, index)
    }
}
