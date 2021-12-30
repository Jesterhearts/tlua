use tlua_bytecode::OpError;
use tlua_parser::ast::statement::{
    Break,
    Empty,
    Goto,
    Label,
};

use crate::{
    CompileError,
    CompileStatement,
    CompilerContext,
};

pub(crate) mod assignment;
pub(crate) mod fn_decl;
pub(crate) mod for_loop;
pub(crate) mod foreach_loop;
pub(crate) mod if_statement;
pub(crate) mod repeat_loop;
pub(crate) mod variables;
pub(crate) mod while_loop;

impl CompileStatement for Empty {
    fn compile(&self, _: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        Ok(None)
    }
}

impl CompileStatement for Break {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}

impl CompileStatement for Label {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}

impl CompileStatement for Goto {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        todo!()
    }
}
