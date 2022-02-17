use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    OpError,
};
use tlua_parser::ast::{
    identifiers::Ident,
    statement::foreach_loop::ForEachLoop,
};

use crate::{
    block::emit_block,
    compiler::{
        InitRegister,
        JumpTemplate,
    },
    statement::assignment::emit_assignments,
    CompileError,
    CompileExpression,
    CompileStatement,
    Scope,
};

impl CompileStatement for ForEachLoop<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let mut scope = scope.new_block();
        let mut scope = scope.enter();
        let loop_exit_label = scope.push_loop_label();

        let (loop_start, pending_skip_body) = emit_loop_header(
            self.vars.iter().copied(),
            self.expressions.iter(),
            &mut scope,
        )?;

        emit_block(&mut scope, &self.body)?;

        scope.emit(opcodes::Jump::from(loop_start));

        pending_skip_body.apply(scope.next_instruction(), &mut scope);

        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}

fn emit_loop_header(
    mut vars: impl ExactSizeIterator<Item = Ident>,
    inits: impl ExactSizeIterator<Item = impl CompileExpression> + Clone,
    scope: &mut Scope,
) -> Result<(usize, JumpTemplate<opcodes::JumpNil>), CompileError> {
    const LOOP_ARGS: usize = 4;
    let var_inits = scope.reserve_immediate_range(LOOP_ARGS + vars.len());
    let mut var_init_regsiters = var_inits.iter();
    let mut scope = guard_on_success(scope, |scope| scope.pop_immediate_range(var_inits));

    let to_be_closed = var_init_regsiters
        .next()
        .expect("At least one control var")
        .no_init_needed();
    let iter_func = var_init_regsiters
        .next()
        .expect("At least one control var")
        .no_init_needed();
    let state = var_init_regsiters
        .next()
        .expect("At least one control var")
        .no_init_needed();
    let control = var_init_regsiters
        .next()
        .expect("At least one control var")
        .no_init_needed();

    let control_vars_list = [iter_func, state, control, to_be_closed];

    emit_assignments(
        &mut scope,
        |_scope, var| Ok(var),
        |scope, var, init| {
            init.into_existing_register(scope, var);
        },
        control_vars_list.into_iter(),
        inits,
    )?;

    let loop_start = scope.emit(opcodes::Call::from((iter_func, usize::from(state), 2)));
    scope.emit(opcodes::ConsumeRetRange::from((
        usize::from(control),
        vars.len(),
    )));

    let named_control = vars.next().expect("At least one named variable");
    scope
        .new_local(named_control)?
        .init_from_immediate(&mut scope, control);
    for (var, reg) in vars.zip(var_init_regsiters) {
        scope
            .new_local(var)?
            .init_from_immediate(&mut scope, reg.no_init_needed());
    }

    Ok((
        loop_start,
        JumpTemplate::<opcodes::JumpNil>::conditional(scope.reserve_jump_isn(), control),
    ))
}
