use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    OpError,
};
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
    CompileExpression,
    CompileStatement,
    NodeOutput,
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
        let mut outputs = self.expressions.iter();
        if outputs.len() == 0 {
            scope.emit(opcodes::Op::Ret);
            return Ok(None);
        }

        let normal_retc = outputs.len() - 1;

        for _ in 0..normal_retc {
            let retval = outputs
                .next()
                .expect("Still in bounds for outputs")
                .compile(scope)?;

            let ret = retval.to_register(scope);
            let mut scope = guard_on_success(&mut *scope, |scope| scope.pop_immediate(ret));
            scope.emit(opcodes::SetRet::from(ret));
        }
        match outputs
            .next()
            .expect("Still in bounds for outputs")
            .compile(scope)?
        {
            NodeOutput::ReturnValues => {
                scope.emit(opcodes::Op::CopyRetFromRetAndRet);
            }
            NodeOutput::VAStack => {
                scope.emit(opcodes::Op::CopyRetFromVaAndRet);
            }
            retval => {
                let ret = retval.to_register(scope);
                let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(ret));
                scope.emit(opcodes::SetRet::from(ret));
                scope.emit(opcodes::Op::Ret);
            }
        }

        debug_assert!(outputs.next().is_none());

        Ok(None)
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
