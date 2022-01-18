use either::Either;
use tlua_bytecode::OpError;
use tlua_parser::ast::statement::assignment::Assignment;

use crate::{
    compiler::{
        unasm::UnasmRegister,
        InitRegister,
    },
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
            &mut prefix_expression::map_var,
            self.varlist.iter(),
            self.expressions.iter(),
        )
    }
}

pub(crate) fn emit_assignments<VarExpr, InVarRegTy: Copy, OutVarRegTy: InitRegister<InVarRegTy>>(
    compiler: &mut CompilerContext,
    compile_var: &mut impl FnMut(
        &mut CompilerContext,
        VarExpr,
    ) -> Result<Either<OutVarRegTy, OpError>, CompileError>,
    mut vars: impl ExactSizeIterator<Item = VarExpr>,
    mut inits: impl ExactSizeIterator<Item = impl CompileExpression>,
) -> Result<Option<OpError>, CompileError>
where
    UnasmRegister: From<InVarRegTy>,
{
    let mut common_length = 0..(vars.len().min(inits.len() - 1));
    while let (Some(_), Some(dest), Some(init)) = (common_length.next(), vars.next(), inits.next())
    {
        let dest = match compile_var(compiler, dest)? {
            Either::Left(reg) => reg,
            Either::Right(err) => return Ok(Some(err)),
        };

        let init = init.compile(compiler)?;
        if let Either::Right(err) = dest.init_from_node_output(compiler, init) {
            return Ok(Some(err));
        }
    }

    if let (Some(dest), Some(init)) = (vars.next(), inits.next()) {
        debug_assert!(vars.next().is_none());
        debug_assert!(inits.next().is_none());

        let dest = match compile_var(compiler, dest)? {
            Either::Left(reg) => reg,
            Either::Right(err) => return Ok(Some(err)),
        };

        let init = init.compile(compiler)?;
        if let Either::Right(err) = dest.init_from_node_output(compiler, init) {
            return Ok(Some(err));
        }
    } else {
        for init in inits {
            if let NodeOutput::Err(err) = init.compile(compiler)? {
                return Ok(Some(err));
            }
        }
    }

    Ok(None)
}
