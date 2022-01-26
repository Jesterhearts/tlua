use tlua_bytecode::OpError;
use tlua_parser::ast::statement::assignment::Assignment;

use crate::{
    compiler::{
        unasm::UnasmRegister,
        InitRegister,
    },
    constant::Constant,
    prefix_expression,
    CompileError,
    CompileExpression,
    CompileStatement,
    CompilerContext,
    NodeOutput,
};

impl CompileStatement for Assignment<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        emit_assignments(
            compiler,
            prefix_expression::map_var,
            self.varlist.iter(),
            self.expressions.iter(),
        )
    }
}

pub(crate) fn emit_assignments<VarExpr, InVarRegTy: Copy, OutVarRegTy: InitRegister<InVarRegTy>>(
    compiler: &mut CompilerContext,
    mut compile_var: impl FnMut(&mut CompilerContext, VarExpr) -> Result<OutVarRegTy, CompileError>,
    mut vars: impl ExactSizeIterator<Item = VarExpr> + Clone,
    mut inits: impl ExactSizeIterator<Item = impl CompileExpression> + Clone,
) -> Result<Option<OpError>, CompileError>
where
    UnasmRegister: From<InVarRegTy>,
{
    let common_length = 0..(vars.len().min(inits.len().saturating_sub(1)));
    for _ in common_length {
        let dest = compile_var(compiler, vars.next().expect("Still in common length"))?;

        let init = inits
            .next()
            .expect("Still in common length")
            .compile(compiler)?;

        dest.init_from_node_output(compiler, init);
    }

    if let Some(dest) = vars.next() {
        let dest = compile_var(compiler, dest)?;

        match inits.next() {
            Some(init) => match init.compile(compiler)? {
                NodeOutput::Constant(value) => {
                    dest.init_from_const(compiler, value);
                }
                NodeOutput::Register(other) => {
                    dest.init_from_reg(compiler, other);
                }
                NodeOutput::ReturnValues => {
                    dest.init_from_ret(compiler);

                    for v in vars {
                        compile_var(compiler, v)?.init_from_ret(compiler);
                    }
                }
                NodeOutput::VAStack => {
                    dest.init_from_va(compiler, 0);

                    for (index, v) in vars.enumerate() {
                        compile_var(compiler, v)?.init_from_va(compiler, index + 1);
                    }
                }
                NodeOutput::Err(err) => return Ok(Some(err)),
            },
            None => {
                dest.init_from_const(compiler, Constant::Nil);
            }
        }

        debug_assert!(inits.next().is_none());
    } else {
        for init in inits {
            init.compile(compiler)?;
        }
    }

    Ok(None)
}
