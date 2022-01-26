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
    CompilerContext,
    NodeOutput,
};

impl CompileStatement for WhileLoop<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        let loop_exit_label = compiler.push_loop_label();

        let cond_start = compiler.next_instruction();
        let init = self.cond.compile(compiler)?;
        let pending_skip_body = compiler.emit(opcodes::Raise {
            err: OpError::ByteCodeError {
                err: ByteCodeError::MissingJump,
                offset: compiler.next_instruction(),
            },
        });

        self.body.compile(compiler)?;
        compiler.emit(opcodes::Jump::from(cond_start));

        let jump_op: UnasmOp = match init {
            NodeOutput::Register(reg) => {
                opcodes::JumpNot::from((reg, compiler.next_instruction())).into()
            }
            NodeOutput::ReturnValues => {
                opcodes::JumpNotRet0::from(compiler.next_instruction()).into()
            }
            NodeOutput::VAStack => opcodes::JumpNotVa0::from(compiler.next_instruction()).into(),
            NodeOutput::Constant(c) => {
                if c.as_bool() {
                    // Infinite loop, no need to jump
                    UnasmOp::Nop
                } else {
                    // Loop never executed, just jump over it.
                    opcodes::Jump::from(compiler.next_instruction()).into()
                }
            }
            NodeOutput::Err(_) => UnasmOp::Nop,
        };

        compiler.overwrite(pending_skip_body, jump_op);

        compiler.label_current_instruction(loop_exit_label)?;
        compiler.pop_loop_label();

        Ok(None)
    }
}
