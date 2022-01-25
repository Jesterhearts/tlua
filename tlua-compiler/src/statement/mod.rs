use tlua_bytecode::{
    opcodes,
    OpError,
};
use tlua_parser::ast::statement::{
    Break,
    Empty,
    Goto,
    Label,
};

use crate::{
    compiler::LabelId,
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
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        match compiler.current_loop_label() {
            Some(label) => {
                compiler.emit_jump_label(label);
                Ok(None)
            }
            None => {
                compiler.emit(opcodes::Raise::from(OpError::BreakNotInLoop));
                Ok(Some(OpError::BreakNotInLoop))
            }
        }
    }
}

impl CompileStatement for Label {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler
            .label_current_instruction(LabelId::Named(self.0))
            .map(|()| None)
    }
}

impl CompileStatement for Goto {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.emit_jump_label(LabelId::Named(self.0));
        Ok(None)
    }
}
