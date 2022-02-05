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
    compiler::{
        unasm::UnasmOp,
        LabelId,
    },
    CompileError,
    CompileExpression,
    CompileStatement,
    CompilerContext,
    NodeOutput,
};

impl CompileStatement for If<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        let exit_label = compiler.create_if_label();

        compile_if_block(compiler, exit_label, &self.cond, &self.body)?;

        for ElseIf { cond, body } in self.elif.iter() {
            compile_if_block(compiler, exit_label, cond, body)?;
        }

        if let Some(else_block) = self.else_final.as_ref() {
            else_block.compile(compiler)?;
        }

        compiler
            .label_current_instruction(exit_label)
            .map(|()| None)
    }
}

fn compile_if_block(
    compiler: &mut CompilerContext,
    exit_label: LabelId,
    cond: &Expression,
    body: &Block,
) -> Result<(), CompileError> {
    let cond_value = cond.compile(compiler)?;
    let cond_reg = compiler.output_to_reg_reuse_anon(cond_value);

    // Reserve an intruction for jumping to the next condition if the operand is
    // false.
    let pending_skip_body = compiler.emit(opcodes::Raise {
        err: OpError::ByteCodeError {
            err: ByteCodeError::MissingJump,
            offset: compiler.next_instruction(),
        },
    });

    body.compile(compiler)?;

    compiler.emit_jump_label(exit_label);

    let jump_op: UnasmOp = match cond_value {
        NodeOutput::Constant(c) => {
            if c.as_bool() {
                // Always true, do nothing and just enter the block
                UnasmOp::Nop
            } else {
                // Always false, jump without examining the condition.
                opcodes::Jump::from(compiler.next_instruction()).into()
            }
        }
        _ => opcodes::JumpNot::from((cond_reg, compiler.next_instruction())).into(),
    };

    // Now that we know how big our body is, we can update our jump instruction for
    // a false condition to move to this location.
    compiler.overwrite(pending_skip_body, jump_op);

    Ok(())
}
