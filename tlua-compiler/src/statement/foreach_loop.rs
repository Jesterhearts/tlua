use tlua_bytecode::{
    opcodes,
    ByteCodeError,
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
        let mut var_inits = scope.new_anon_reg_range(LOOP_ARGS + vars.len());

        let to_be_closed = var_inits
            .next()
            .expect("At least one control var")
            .no_init_needed();

        let iter_func = var_inits
            .next()
            .expect("At least one control var")
            .no_init_needed();

        let state = var_inits
            .next()
            .expect("At least one control var")
            .no_init_needed();

        let control = var_inits
            .next()
            .expect("At least one control var")
            .no_init_needed();

        // Lua's ordering for control vars is different from ours, so we re-order here
        let control_vars_list = [iter_func, state, control, to_be_closed];
        emit_assignments(
            &mut scope,
            |_scope, var| Ok(var),
            |scope, var, init| {
                var.init_from_node_output(scope, init);
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

        for (var, reg) in vars.zip(var_inits) {
            scope
                .new_local(var)?
                .init_from_anon_reg(&mut scope, reg.no_init_needed());
        }

        let pending_skip_body = scope.emit(opcodes::Raise {
            err: OpError::ByteCodeError {
                err: ByteCodeError::MissingJump,
                offset: scope.next_instruction(),
            },
        });

        emit_block(&mut scope, &self.body)?;

        scope.emit(opcodes::Jump::from(loop_start));

        scope.overwrite(
            pending_skip_body,
            opcodes::JumpNil::from((control, scope.next_instruction())),
        );
        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}
