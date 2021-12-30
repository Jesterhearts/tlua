use tlua_bytecode::OpError;
use tlua_parser::ast::statement::fn_decl::FnDecl;

use crate::{
    expressions::function_defs::compile_global_fn_body,
    CompileError,
    CompileStatement,
    CompilerContext,
};

impl CompileStatement for FnDecl<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        match self {
            FnDecl::Function { body, .. } => {
                let _func_id = compile_global_fn_body(body, compiler)?;
                todo!()
            }
            FnDecl::Local { body, name } => {
                let () = if body.params.varargs {
                    compiler.write_va_local_fn(
                        *name,
                        body.params.named_params.iter().copied(),
                        body.body.statements.iter(),
                        body.body.ret.as_ref(),
                    )?
                } else {
                    compiler.write_local_fn(
                        *name,
                        body.params.named_params.iter().copied(),
                        body.body.statements.iter(),
                        body.body.ret.as_ref(),
                    )?
                };
            }
        };

        Ok(None)
    }
}
