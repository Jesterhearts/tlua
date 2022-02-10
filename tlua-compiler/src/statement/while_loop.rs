use tlua_bytecode::{
    opcodes,
    ByteCodeError,
    OpError,
    Truthy,
};
use tlua_parser::ast::statement::while_loop::WhileLoop;

use crate::{
    compiler::unasm::UnasmOp,
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
        let cond = scope.output_to_reg_reuse_anon(init);

        let pending_skip_body = scope.emit(opcodes::Raise {
            err: OpError::ByteCodeError {
                err: ByteCodeError::MissingJump,
                offset: scope.next_instruction(),
            },
        });

        self.body.compile(scope)?;
        scope.emit(opcodes::Jump::from(cond_start));

        let jump_op: UnasmOp = match init {
            NodeOutput::Constant(c) => {
                if c.as_bool() {
                    // Infinite loop, no need to jump
                    UnasmOp::Nop
                } else {
                    // Loop never executed, just jump over it.
                    opcodes::Jump::from(scope.next_instruction()).into()
                }
            }
            _ => opcodes::JumpNot::from((cond, scope.next_instruction())).into(),
        };

        scope.overwrite(pending_skip_body, jump_op);

        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}
