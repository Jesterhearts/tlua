use either::Either;
use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    Constant,
    OpError,
};
use tlua_parser::statement::assignment::Assignment;

use crate::{
    compiler::RegisterOps,
    prefix_expression::{
        self,
        TableIndex,
    },
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
    Scope,
};

impl CompileStatement for Assignment<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        emit_assignments(
            scope,
            prefix_expression::map_var,
            |scope, dest, init| match dest {
                Either::Left(var) => {
                    let init = init.into_register(scope);
                    let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(init));
                    var.set_from_immediate(&mut scope, init)
                }
                Either::Right(TableIndex { table, index }) => {
                    let init = init.into_register(scope);
                    let mut scope = guard_on_success(scope, |scope| scope.pop_immediate(table));
                    let mut scope =
                        guard_on_success(&mut scope, |scope| scope.pop_immediate(index));
                    let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(init));

                    scope.emit(opcodes::SetProperty::from((table, index, init)));
                    Ok(())
                }
            },
            self.varlist.iter(),
            self.expressions.iter(),
        )
    }
}

pub(crate) fn emit_assignments<VarExpr, VarDest>(
    scope: &mut Scope,
    mut compile_var: impl FnMut(&mut Scope, VarExpr) -> Result<VarDest, CompileError>,
    mut assign_var: impl FnMut(&mut Scope, VarDest, NodeOutput) -> Result<(), CompileError>,
    mut vars: impl ExactSizeIterator<Item = VarExpr> + Clone,
    mut inits: impl ExactSizeIterator<Item = impl CompileExpression> + Clone,
) -> Result<Option<OpError>, CompileError> {
    let common_length = 0..(vars.len().min(inits.len().saturating_sub(1)));
    for _ in common_length {
        let dest = compile_var(scope, vars.next().expect("Still in common length"))?;

        let init = inits
            .next()
            .expect("Still in common length")
            .compile(scope)?;

        assign_var(scope, dest, init)?;
    }

    if let Some(dest) = vars.next() {
        let dest = compile_var(scope, dest)?;

        match inits.next() {
            Some(init) => {
                let init = init.compile(scope)?;

                match init {
                    NodeOutput::ReturnValues => {
                        let consumed_values = vars.len() + 1;
                        let mut regs = scope.reserve_immediate_range(consumed_values).iter();

                        let first = regs.next().expect("At least one var.");
                        scope.emit(opcodes::ConsumeRetRange::from((
                            usize::from(first),
                            consumed_values,
                        )));

                        assign_var(scope, dest, NodeOutput::Immediate(first))?;
                        for (dest, reg) in vars.zip(regs) {
                            let dest = compile_var(scope, dest)?;
                            assign_var(scope, dest, NodeOutput::Immediate(reg))?;
                        }
                    }
                    NodeOutput::VAStack => {
                        let consumed_values = vars.len() + 1;
                        let mut regs = scope.reserve_immediate_range(consumed_values).iter();

                        let first = regs.next().expect("At least one var.");
                        scope.emit(opcodes::LoadVa::from((
                            usize::from(first),
                            0,
                            consumed_values,
                        )));

                        assign_var(scope, dest, NodeOutput::Immediate(first))?;
                        for (dest, reg) in vars.zip(regs) {
                            let dest = compile_var(scope, dest)?;
                            assign_var(scope, dest, NodeOutput::Immediate(reg))?;
                        }
                    }
                    init => {
                        assign_var(scope, dest, init)?;
                    }
                }
            }
            None => {
                assign_var(scope, dest, NodeOutput::Constant(Constant::Nil))?;
            }
        }

        debug_assert!(inits.next().is_none());
    } else {
        for init in inits {
            init.compile(scope)?;
        }
    }

    Ok(None)
}
