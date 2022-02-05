use tlua_parser::ast::expressions::function_defs::FnBody;

use crate::{
    compiler::{
        HasVaArgs,
        InitRegister,
    },
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for FnBody<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let func_id = compiler.emit_fn(
            if self.params.varargs {
                HasVaArgs::Some
            } else {
                HasVaArgs::None
            },
            self.params.named_params.iter().copied(),
            self.body.statements.iter(),
            self.body.ret.as_ref(),
        )?;

        Ok(NodeOutput::Immediate(
            compiler.new_anon_reg().init_alloc_fn(compiler, func_id),
        ))
    }
}
