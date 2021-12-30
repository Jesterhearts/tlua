use tlua_bytecode::FuncId;
use tlua_parser::ast::expressions::function_defs::FnBody;

use crate::{
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

pub(crate) fn compile_global_fn_body(
    body: &FnBody,
    compiler: &mut CompilerContext,
) -> Result<FuncId, CompileError> {
    if body.params.varargs {
        compiler.write_va_global_fn(
            body.params.named_params.iter().copied(),
            body.body.statements.iter(),
            body.body.ret.as_ref(),
        )
    } else {
        compiler.write_global_fn(
            body.params.named_params.iter().copied(),
            body.body.statements.iter(),
            body.body.ret.as_ref(),
        )
    }
}

impl CompileExpression for FnBody<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        let func_id = compile_global_fn_body(self, compiler)?;
        Ok(NodeOutput::Function(func_id))
    }
}
