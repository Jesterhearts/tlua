use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    OpError,
};
use tlua_parser::ast::statement::foreach_loop::ForEachLoop;

use crate::{
    block::emit_block,
    compiler::InitRegister,
    statement::assignment::emit_assignments,
    CompileError,
    CompileStatement,
    Scope,
};

impl CompileStatement for ForEachLoop<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let mut scope = scope.new_block();
        let mut scope = scope.enter();
        let loop_exit_label = scope.push_loop_label();

        let mut vars = self.vars.iter().copied();

        const LOOP_ARGS: usize = 4;
        // We store the registers in this order:
        //  [to_be_closed, iter_func, state, control, ...]
        // This allows us to group everything together in one block, and makes it so
        // calls and returns can slice-assign arguments/return values.
        // to_be_closed is never written to by the loop, so we don't really care where
        // it lives.
        let var_inits = scope.reserve_anon_reg_range(LOOP_ARGS + vars.len());
        let mut var_init_regsiters = var_inits.iter();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg_range(var_inits));

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

        // Lua's ordering for control vars is different from ours, so we re-order here
        let control_vars_list = [iter_func, state, control, to_be_closed];
        emit_assignments(
            &mut scope,
            |_scope, var| Ok(var),
            |scope, var, init| {
                let init = init.to_register(scope);
                let mut scope = guard_on_success(scope, |scope| scope.pop_anon_reg(init));
                var.init_from_anon_reg(&mut scope, init);
            },
            control_vars_list.into_iter(),
            self.expressions.iter(),
        )?;

        // call iter_func(state, control).
        let loop_start = scope.emit(opcodes::Call::from((iter_func, usize::from(state), 2)));

        // control, ... = call...
        scope.emit(opcodes::ConsumeRetRange::from((
            usize::from(control),
            vars.len(),
        )));

        let named_control = vars.next().expect("At least one named variable");
        scope
            .new_local(named_control)?
            .init_from_anon_reg(&mut scope, control);

        for (var, reg) in vars.zip(var_init_regsiters) {
            scope
                .new_local(var)?
                .init_from_anon_reg(&mut scope, reg.no_init_needed());
        }

        let pending_skip_body = scope.reserve_jump_isn();

        emit_block(&mut scope, &self.body)?;

        scope.emit(opcodes::Jump::from(loop_start));

        let next_isn = scope.next_instruction();
        scope.overwrite(
            pending_skip_body,
            opcodes::JumpNil::from((control, next_isn)),
        );
        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}
