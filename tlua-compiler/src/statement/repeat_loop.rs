use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    OpError,
    Truthy,
};
use tlua_parser::ast::statement::repeat_loop::RepeatLoop;

use crate::{
    block::emit_block,
    compiler::unasm::UnasmOp,
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
    Scope,
};

impl CompileStatement for RepeatLoop<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let loop_exit_label = scope.push_loop_label();
        let block_start = scope.next_instruction();

        let mut scope = scope.new_block();
        let mut scope = scope.enter();

        emit_block(&mut scope, &self.body)?;

        let cond = self.terminator.compile(&mut scope)?;

        let jump_op: UnasmOp = match cond {
            NodeOutput::Constant(c) => {
                if c.as_bool() {
                    // Loop immediately terminates, no need to jump
                    UnasmOp::Nop
                } else {
                    // Infinite loop, no need to evaluate op
                    opcodes::Jump::from(block_start).into()
                }
            }
            cond => {
                let cond_reg = cond.into_register(&mut scope);
                let _scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(cond_reg));
                opcodes::JumpNot::from((cond_reg, block_start)).into()
            }
        };
        scope.emit(jump_op);

        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}
