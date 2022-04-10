use tlua_bytecode::{
    opcodes,
    Constant,
    ImmediateRegister,
};

use crate::{
    compiler::{
        unasm::MappedLocalRegister,
        Scope,
    },
    BuiltinType,
    CompileError,
    FuncId,
    Void,
};

pub(crate) trait RegisterOps {
    type Err: Into<CompileError>;

    fn set_from_ret(&self, scope: &mut Scope) -> Result<(), Self::Err>;

    fn set_from_constant(&self, scope: &mut Scope, value: Constant) -> Result<(), Self::Err>;

    fn alloc_and_set_from_fn(&self, scope: &mut Scope, value: FuncId) -> Result<(), Self::Err>;

    fn alloc_and_set_from_table(&self, scope: &mut Scope) -> Result<(), Self::Err>;

    fn set_from_immediate(
        &self,
        scope: &mut Scope,
        other: ImmediateRegister,
    ) -> Result<(), Self::Err>;

    fn set_from_local(
        &self,
        scope: &mut Scope,
        other: MappedLocalRegister,
    ) -> Result<(), Self::Err>;

    fn set_from_table_entry(
        &self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> Result<(), Self::Err>;

    fn set_from_va(&self, scope: &mut Scope, index: usize) -> Result<(), Self::Err>;
}

impl RegisterOps for ImmediateRegister {
    type Err = Void;

    fn set_from_ret(&self, scope: &mut Scope) -> Result<(), Self::Err> {
        scope.emit(opcodes::ConsumeRetRange::from((usize::from(*self), 1)));
        Ok(())
    }

    fn set_from_constant(&self, scope: &mut Scope, value: Constant) -> Result<(), Self::Err> {
        scope.emit(opcodes::LoadConstant::from((*self, value)));
        Ok(())
    }

    fn alloc_and_set_from_fn(&self, scope: &mut Scope, value: FuncId) -> Result<(), Self::Err> {
        scope.emit(opcodes::Alloc::from((
            *self,
            BuiltinType::Function(value).into(),
        )));
        Ok(())
    }

    fn alloc_and_set_from_table(&self, scope: &mut Scope) -> Result<(), Self::Err> {
        scope.emit(opcodes::Alloc::from((*self, BuiltinType::Table.into())));
        Ok(())
    }

    fn set_from_immediate(
        &self,
        scope: &mut Scope,
        other: ImmediateRegister,
    ) -> Result<(), Self::Err> {
        if other != *self {
            scope.emit(opcodes::DuplicateRegister::from((*self, other)));
        }
        Ok(())
    }

    fn set_from_local(
        &self,
        scope: &mut Scope,
        other: MappedLocalRegister,
    ) -> Result<(), Self::Err> {
        scope.emit(opcodes::LoadRegister::from((*self, other)));
        Ok(())
    }

    fn set_from_table_entry(
        &self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> Result<(), Self::Err> {
        scope.emit(opcodes::Lookup::from((*self, table, index)));
        Ok(())
    }

    fn set_from_va(&self, scope: &mut Scope, index: usize) -> Result<(), Self::Err> {
        scope.emit(opcodes::LoadVa::from((usize::from(*self), index, 1)));
        Ok(())
    }
}

impl RegisterOps for MappedLocalRegister {
    type Err = CompileError;

    fn set_from_ret(&self, scope: &mut Scope) -> Result<(), CompileError> {
        let reg = scope.push_immediate();
        reg.set_from_ret(scope)?;
        scope.emit(opcodes::Store::from((*self, reg)));
        scope.pop_immediate(reg);
        Ok(())
    }

    fn set_from_constant(&self, scope: &mut Scope, value: Constant) -> Result<(), CompileError> {
        let reg = scope.push_immediate();
        reg.set_from_constant(scope, value)?;
        scope.emit(opcodes::Store::from((*self, reg)));
        scope.pop_immediate(reg);
        Ok(())
    }

    fn alloc_and_set_from_fn(&self, scope: &mut Scope, value: FuncId) -> Result<(), CompileError> {
        let reg = scope.push_immediate();
        reg.alloc_and_set_from_fn(scope, value)?;
        scope.emit(opcodes::Store::from((*self, reg)));
        scope.pop_immediate(reg);
        Ok(())
    }

    fn alloc_and_set_from_table(&self, scope: &mut Scope) -> Result<(), CompileError> {
        let reg = scope.push_immediate();
        reg.alloc_and_set_from_table(scope)?;
        scope.emit(opcodes::Store::from((*self, reg)));
        scope.pop_immediate(reg);
        Ok(())
    }

    fn set_from_immediate(
        &self,
        scope: &mut Scope,
        other: ImmediateRegister,
    ) -> Result<(), CompileError> {
        scope.emit(opcodes::Store::from((*self, other)));
        Ok(())
    }

    fn set_from_local(
        &self,
        scope: &mut Scope,
        other: MappedLocalRegister,
    ) -> Result<(), CompileError> {
        if other != *self {
            let reg = scope.push_immediate();
            reg.set_from_local(scope, other)?;
            scope.emit(opcodes::Store::from((*self, reg)));
        }
        Ok(())
    }

    fn set_from_table_entry(
        &self,
        scope: &mut Scope,
        table: ImmediateRegister,
        index: ImmediateRegister,
    ) -> Result<(), CompileError> {
        let reg = scope.push_immediate();
        reg.set_from_table_entry(scope, table, index)?;
        scope.emit(opcodes::Store::from((*self, reg)));
        Ok(())
    }

    fn set_from_va(&self, scope: &mut Scope, index: usize) -> Result<(), CompileError> {
        let reg = scope.push_immediate();
        reg.set_from_va(scope, index)?;
        scope.emit(opcodes::Store::from((*self, reg)));
        Ok(())
    }
}
