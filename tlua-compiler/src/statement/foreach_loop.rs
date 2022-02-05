use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
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

            let (control_vars_start, control_vars_list) = compiler.new_anon_reg_range(4);

            // `state` in the list below.
            let iter_func_args_start = control_vars_start + 1;
            let [iter_func, _state, anon_control, _to_be_closed] = {
                let mut control_vars = [AnonymousRegister::from(0); 4];
                for (var, reg) in control_vars.iter_mut().zip(control_vars_list.clone()) {
                    // Will be initialized below.
                    *var = reg.no_init_needed();
                }
                control_vars
            };

            emit_assignments(
                compiler,
                |_compiler, var| Ok(var),
                |compiler, var, init| {
                    var.init_from_node_output(compiler, init);
                },
                control_vars_list,
                self.expressions.iter(),
            )?;

            // iter_func(state, control_init).
            let loop_start =
                compiler.emit(opcodes::Call::from((iter_func, iter_func_args_start, 2)));

            let mut vars = self.vars.iter().copied();
            let named_control = vars.next().expect("At least one named variable");

            anon_control.init_from_ret(compiler);

            for var in vars {
                compiler.new_local(var)?.init_from_ret(compiler);
            }

            compiler
                .new_local(named_control)?
                .init_from_anon_reg(compiler, anon_control);

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
                opcodes::JumpNil::from((anon_control, compiler.next_instruction())),
            );
            compiler.label_current_instruction(loop_exit_label)?;
            compiler.pop_loop_label();

            Ok(None)
        })
    }
}
