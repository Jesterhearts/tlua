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
    CompilerContext,
};

impl CompileStatement for ForEachLoop<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.emit_in_subscope(|compiler| {
            let loop_exit_label = compiler.push_loop_label();

            let mut vars = self.vars.iter().copied();

            const LOOP_ARGS: usize = 4;
            // We store the registers in this order:
            //  [to_be_closed, iter_func, state, control, ...]
            // This allows us to group everything together in one block, and makes it so
            // calls and returns can slice-assign arguments/return values.
            // to_be_closed is never written to by the loop, so we don't really care where
            // it lives.
            let mut var_inits = compiler.new_anon_reg_range(LOOP_ARGS + vars.len());

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
                compiler,
                |_compiler, var| Ok(var),
                |compiler, var, init| {
                    var.init_from_node_output(compiler, init);
                },
                control_vars_list.into_iter(),
                self.expressions.iter(),
            )?;

            // call iter_func(state, control).
            let loop_start = compiler.emit(opcodes::Call::from((iter_func, usize::from(state), 2)));

            // control, ... = call...
            compiler.emit(opcodes::ConsumeRetRange::from((
                usize::from(control),
                vars.len(),
            )));

            let named_control = vars.next().expect("At least one named variable");
            compiler
                .new_local(named_control)?
                .init_from_anon_reg(compiler, control);

            for (var, reg) in vars.zip(var_inits) {
                compiler
                    .new_local(var)?
                    .init_from_anon_reg(compiler, reg.no_init_needed());
            }

            let pending_skip_body = compiler.emit(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: compiler.next_instruction(),
                },
            });

            emit_block(compiler, &self.body)?;

            compiler.emit(opcodes::Jump::from(loop_start));

            compiler.overwrite(
                pending_skip_body,
                opcodes::JumpNil::from((control, compiler.next_instruction())),
            );
            compiler.label_current_instruction(loop_exit_label)?;
            compiler.pop_loop_label();

            Ok(None)
        })
    }
}
