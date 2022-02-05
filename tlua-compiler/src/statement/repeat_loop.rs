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
    CompilerContext,
    NodeOutput,
};

impl CompileStatement for RepeatLoop<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        let loop_exit_label = compiler.push_loop_label();
        let block_start = compiler.next_instruction();

        compiler.emit_in_subscope(|compiler| {
            emit_block(compiler, &self.body)?;

            let cond = self.terminator.compile(compiler)?;
            let cond_reg = compiler.output_to_reg_reuse_anon(cond);

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
                _ => opcodes::JumpNot::from((cond_reg, block_start)).into(),
            };
            compiler.emit(jump_op);

            Ok(None)
        })?;

        compiler.label_current_instruction(loop_exit_label)?;
        compiler.pop_loop_label();

        Ok(None)
    }
}
