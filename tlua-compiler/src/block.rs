use tlua_bytecode::{
    opcodes,
    ByteCodeError,
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
        compiler.emit_in_subscope(|compiler| {
            let pending_scope_push = compiler.emit(opcodes::Raise::from(OpError::ByteCodeError {
                err: ByteCodeError::MissingScopeDescriptor,
                offset: compiler.current_instruction(),
            }));

            for stat in self.statements.iter() {
                if let Some(err) = stat.compile(compiler)? {
                    return Ok(Some(err));
                }
            }

            compiler.overwrite(
                pending_scope_push,
                opcodes::ScopeDescriptor::from(compiler.scope_declared_locals_count()),
            );

            match self.ret.as_ref() {
                Some(ret) => ret.compile(compiler),
                None => {
                    compiler.emit(opcodes::Op::PopScope);
                    Ok(None)
                }
            }
        })
    }
}
