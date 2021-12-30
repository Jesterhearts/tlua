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
        compiler.write_subscope(self.statements.iter(), self.ret.as_ref())
    }
}
