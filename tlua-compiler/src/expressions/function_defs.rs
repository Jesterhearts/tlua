use tlua_bytecode::opcodes;
use tlua_parser::{
    expressions::function_defs::FnBody,
    identifiers::Ident,
};

use crate::{
    compiler::{
        HasVaArgs,
        InitRegister,
    },
    CompileError,
    CompileExpression,
    CompileStatement,
    FuncId,
    NodeOutput,
    Scope,
};

pub(crate) fn emit_fn(
    scope: &mut Scope,
    has_va_args: HasVaArgs,
    is_method: bool,
    params: impl ExactSizeIterator<Item = Ident>,
    body: impl ExactSizeIterator<Item = impl CompileStatement>,
    ret: Option<&impl CompileStatement>,
) -> Result<FuncId, CompileError> {
    let mut func = scope.new_function(has_va_args, params.len() + usize::from(is_method));
    {
        let mut scope = func.start();
        let mut scope = scope.enter();

        if is_method {
            scope.new_local_self()?.no_init_needed();
        }

        for param in params {
            // TODO(compiler-opt): Technically today this allocates an extra, unused
            // register for every duplicate identifier in the parameter list. It
            // still works fine though, because the number of registers is
            // correct.
            scope.new_local(param)?.no_init_needed();
        }

        for stat in body {
            stat.compile(&mut scope)?;
        }

        match ret {
            Some(ret) => ret.compile(&mut scope)?,
            None => {
                scope.emit(opcodes::Op::Ret);
                None
            }
        };
    }

    Ok(func.complete())
}

impl CompileExpression for FnBody<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        let func_id = emit_fn(
            scope,
            if self.params.varargs {
                HasVaArgs::Some
            } else {
                HasVaArgs::None
            },
            false,
            self.params.named_params.iter().copied(),
            self.body.statements.iter(),
            self.body.ret.as_ref(),
        )?;

        Ok(NodeOutput::Immediate(
            scope.push_immediate().init_alloc_fn(scope, func_id),
        ))
    }
}
