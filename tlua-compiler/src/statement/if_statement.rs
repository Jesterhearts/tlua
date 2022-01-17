use tlua_bytecode::{
    opcodes,
    ByteCodeError,
    OpError,
    Truthy,
};
use tlua_parser::ast::{
    block::Block,
    expressions::Expression,
    statement::if_statement::{
        ElseIf,
        If,
    },
};

use crate::{
    compiler::unasm::UnasmOp,
    CompileError,
    CompileExpression,
    CompileStatement,
    CompilerContext,
    NodeOutput,
};

impl CompileStatement for If<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        // We daisy-chain block exits for simplicity, so each block jumps to the the
        // exit instruction of the next block in the chain.
        let mut pending_exit = compile_if_block(compiler, &self.cond, &self.body)?;

        for ElseIf { cond, body } in self.elif.iter() {
            let block_exit = compile_if_block(compiler, cond, body)?;

            compiler.overwrite(pending_exit, opcodes::Jump::from(block_exit));
            pending_exit = block_exit;
        }

        if let Some(else_block) = self.else_final.as_ref() {
            else_block.compile(compiler)?;
            compiler.overwrite(
                pending_exit,
                opcodes::Jump::from(compiler.current_instruction()),
            );
        } else {
            // Whatever if/elif block was last evaluated is the last in the sequence, we
            // don't need to jump out of it since there's no trailing else and can just nop.
            compiler.overwrite(pending_exit, UnasmOp::Nop);
        }

        Ok(None)
    }
}

fn compile_if_block(
    compiler: &mut CompilerContext,
    cond: &Expression,
    body: &Block,
) -> Result<usize, CompileError> {
    let cond_value = cond.compile(compiler)?;

    // Reserve an intruction for jumping to the next condition if the operand is
    // false.
    let pending_skip_body = compiler.emit(opcodes::Raise {
        err: OpError::ByteCodeError {
            err: ByteCodeError::MissingJump,
            offset: compiler.current_instruction(),
        },
    });

    body.compile(compiler)?;

    // Reserve an instruction for jumping out of the if sequence after evaluating
    // the body.
    let block_exit = compiler.emit(opcodes::Raise {
        err: OpError::ByteCodeError {
            err: ByteCodeError::MissingJump,
            offset: compiler.current_instruction(),
        },
    });

    let jump_op: UnasmOp = match cond_value {
        NodeOutput::Register(reg) => {
            opcodes::JumpNot::from((reg, compiler.current_instruction())).into()
        }
        NodeOutput::ReturnValues => {
            opcodes::JumpNotRet0::from(compiler.current_instruction()).into()
        }
        NodeOutput::VAStack => opcodes::JumpNotVa0::from(compiler.current_instruction()).into(),
        NodeOutput::Constant(c) => {
            if c.as_bool() {
                // Always true, do nothing and just enter the block
                UnasmOp::Nop
            } else {
                // Always false, jump without examining the condition.
                opcodes::Jump::from(compiler.current_instruction()).into()
            }
        }
        NodeOutput::Err(_) => UnasmOp::Nop,
    };

    // Now that we know how big our body is, we can update our jump instruction for
    // a false condition to move to this location.
    compiler.overwrite(pending_skip_body, jump_op);

    Ok(block_exit)
}
