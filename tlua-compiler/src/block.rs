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
    CompilerContext,
};

impl CompileStatement for List<'_, Statement<'_>> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        for stat in self.iter() {
            stat.compile(compiler)?;
        }

        Ok(None)
    }
}

impl CompileStatement for RetStatement<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.write_ret_stack_sequence(self.expressions.iter())
    }
}

impl CompileStatement for Block<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.emit_in_subscope(|compiler| emit_block(compiler, self))
    }
}

pub(crate) fn emit_block(
    compiler: &mut CompilerContext,
    block: &Block,
) -> Result<Option<OpError>, CompileError> {
    for stat in block.statements.iter() {
        stat.compile(compiler)?;
    }

    match block.ret.as_ref() {
        Some(ret) => ret.compile(compiler),
        None => Ok(None),
    }
}
