use either::Either;
use tlua_bytecode::{
    opcodes,
    OpError,
};
use tlua_parser::ast::statement::assignment::Assignment;

use crate::{
    compiler::InitRegister,
    constant::Constant,
    prefix_expression::{
        self,
        TableIndex,
    },
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
            |compiler, dest, init| match dest {
                Either::Left(var) => {
                    var.init_from_node_output(compiler, init);
                }
                Either::Right(TableIndex { table, index }) => {
                    let init = compiler.output_to_reg_reuse_anon(init);
                    compiler.emit(opcodes::SetProperty::from((table, index, init)));
                }
            },
            self.varlist.iter(),
            self.expressions.iter(),
        )
    }
}

pub(crate) fn emit_assignments<VarExpr, VarDest>(
    compiler: &mut CompilerContext,
    mut compile_var: impl FnMut(&mut CompilerContext, VarExpr) -> Result<VarDest, CompileError>,
    mut assign_var: impl FnMut(&mut CompilerContext, VarDest, NodeOutput),
    mut vars: impl ExactSizeIterator<Item = VarExpr> + Clone,
    mut inits: impl ExactSizeIterator<Item = impl CompileExpression> + Clone,
) -> Result<Option<OpError>, CompileError> {
    let common_length = 0..(vars.len().min(inits.len().saturating_sub(1)));
    for _ in common_length {
        let dest = compile_var(compiler, vars.next().expect("Still in common length"))?;

        let init = inits
            .next()
            .expect("Still in common length")
            .compile(compiler)?;

        assign_var(compiler, dest, init);
    }

    if let Some(dest) = vars.next() {
        let dest = compile_var(compiler, dest)?;

        match inits.next() {
            Some(init) => {
                let init = init.compile(compiler)?;
                assign_var(compiler, dest, init);

                match init {
                    init @ NodeOutput::ReturnValues | init @ NodeOutput::VAStack => {
                        for dest in vars {
                            let dest = compile_var(compiler, dest)?;
                            assign_var(compiler, dest, init)
                        }
                    }
                    _ => (),
                }
            }
            None => {
                assign_var(compiler, dest, NodeOutput::Constant(Constant::Nil));
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
