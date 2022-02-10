use tlua_bytecode::OpError;
use tlua_parser::{
    ast::{
        block::{
            retstat::RetStatement,
            Block,
        },
        statement::Statement,
    },
    list::List,
};

use crate::{
    CompileError,
    CompileStatement,
    Scope,
};

impl CompileStatement for List<'_, Statement<'_>> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        for stat in self.iter() {
            stat.compile(scope)?;
        }

        Ok(None)
    }
}

impl CompileStatement for RetStatement<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        scope.write_ret_stack_sequence(self.expressions.iter())
    }
}

impl CompileStatement for Block<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let mut scope = scope.new_block();
        let mut scope = scope.enter();
        emit_block(&mut scope, self)
    }
}

pub(crate) fn emit_block(
    scope: &mut Scope,
    block: &Block,
) -> Result<Option<OpError>, CompileError> {
    for stat in block.statements.iter() {
        stat.compile(scope)?;
    }

    match block.ret.as_ref() {
        Some(ret) => ret.compile(scope),
        None => Ok(None),
    }
}
