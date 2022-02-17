use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    OpError,
    Truthy,
};
use tlua_parser::ast::statement::while_loop::WhileLoop;

use crate::{
    compiler::JumpTemplate,
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
    Scope,
};

impl CompileStatement for WhileLoop<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let loop_exit_label = scope.push_loop_label();

        let cond_start = scope.next_instruction();
        let init = self.cond.compile(scope)?;

        let pending_skip_body = match init {
            NodeOutput::Constant(c) => {
                if c.as_bool() {
                    // Infinite loop, no need to jump
                    None
                } else {
                    // Loop never executed, just jump over it.
                    Some(JumpTemplate::unconditional_at(scope.reserve_jump_isn()))
                }
            }
            init => {
                let cond = init.into_register(scope);
                let mut scope = guard_on_success(&mut *scope, |scope| scope.pop_immediate(cond));
                Some(JumpTemplate::<opcodes::JumpNot>::conditional_at(
                    scope.reserve_jump_isn(),
                    cond,
                ))
            }
        };

        self.body.compile(scope)?;
        scope.emit(opcodes::Jump::from(cond_start));

        if let Some(jump) = pending_skip_body {
            jump.resolve_to(scope.next_instruction(), scope)
        }

        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}
