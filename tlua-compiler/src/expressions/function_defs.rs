use tlua_bytecode::TypeMeta;
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

pub(crate) fn compile_global_fn_body(
    body: &FnBody,
    compiler: &mut CompilerContext,
) -> Result<TypeMeta, CompileError> {
    let mut context = compiler.function_subcontext(if body.params.varargs {
        HasVaArgs::Some
    } else {
        HasVaArgs::None
    });

    context.emit_fn(
        body.params.named_params.iter().copied(),
        body.body.statements.iter(),
        body.body.ret.as_ref(),
    )?;

    Ok(context.complete_subcontext())
}

impl CompileExpression for FnBody<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let func_id = compile_global_fn_body(self, compiler)?;
        Ok(NodeOutput::Register(
            compiler
                .new_anon_reg()
                .init_alloc_fn(compiler, func_id)
                .into(),
        ))
    }
}
