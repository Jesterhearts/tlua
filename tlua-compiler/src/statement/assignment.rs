use either::Either;
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
    mut compile_var: impl FnMut(
        &mut CompilerContext,
        VarExpr,
    ) -> Result<Either<OutVarRegTy, OpError>, CompileError>,
    mut vars: impl ExactSizeIterator<Item = VarExpr> + Clone,
    mut inits: impl ExactSizeIterator<Item = impl CompileExpression> + Clone,
) -> Result<Option<OpError>, CompileError>
where
    UnasmRegister: From<InVarRegTy>,
{
    let common_length = 0..(vars.len().min(inits.len().saturating_sub(1)));
    for _ in common_length {
        let dest = match compile_var(compiler, vars.next().expect("Still in common length"))? {
            Either::Left(reg) => reg,
            Either::Right(err) => return Ok(Some(err)),
        };

        let init = inits
            .next()
            .expect("Still in common length")
            .compile(compiler)?;
        if let Either::Right(err) = dest.init_from_node_output(compiler, init) {
            return Ok(Some(err));
        }
    }

    if let Some(dest) = vars.next() {
        let dest = match compile_var(compiler, dest)? {
            Either::Left(reg) => reg,
            Either::Right(err) => return Ok(Some(err)),
        };

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
                        match compile_var(compiler, v)? {
                            Either::Left(reg) => {
                                reg.init_from_ret(compiler);
                            }
                            Either::Right(err) => return Ok(Some(err)),
                        };
                    }
                }
                NodeOutput::VAStack => {
                    dest.init_from_va(compiler, 0);

                    for (index, v) in vars.enumerate() {
                        match compile_var(compiler, v)? {
                            Either::Left(reg) => {
                                reg.init_from_va(compiler, index + 1);
                            }
                            Either::Right(err) => return Ok(Some(err)),
                        };
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
            if let NodeOutput::Err(err) = init.compile(compiler)? {
                return Ok(Some(err));
            }
        }
    }

    Ok(None)
}
