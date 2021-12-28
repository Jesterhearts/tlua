use tlua_parser::ast::statement::{
    Break,
    Empty,
    Goto,
    Label,
};

use crate::{
    compiling::{
        CompileError,
        CompileStatement,
        CompilerContext,
    },
    vm::OpError,
};

pub mod assignment;
pub mod fn_decl;
pub mod for_loop;
pub mod foreach_loop;
pub mod if_statement;
pub mod repeat_loop;
pub mod variables;
pub mod while_loop;

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
