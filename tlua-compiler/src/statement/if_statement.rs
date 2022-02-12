use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
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
    NodeOutput,
    Scope,
};

impl CompileStatement for If<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let exit_label = scope.create_if_label();

        compile_if_block(scope, exit_label, &self.cond, &self.body)?;

        for ElseIf { cond, body } in self.elif.iter() {
            compile_if_block(scope, exit_label, cond, body)?;
        }

        if let Some(else_block) = self.else_final.as_ref() {
            else_block.compile(scope)?;
        }

        scope.label_current_instruction(exit_label).map(|()| None)
    }
}

fn compile_if_block(
    scope: &mut Scope,
    exit_label: LabelId,
    cond: &Expression,
    body: &Block,
) -> Result<(), CompileError> {
    let cond_value = cond.compile(scope)?;
    let cond_reg = cond_value.to_register(scope);
    let mut scope = guard_on_success(scope, |scope| scope.pop_anon_reg(cond_reg));

    // Reserve an intruction for jumping to the next condition if the operand is
    // false.
    let pending_skip_body = scope.reserve_jump_isn();

    body.compile(&mut scope)?;

    scope.emit_jump_label(exit_label);

    let jump_op: UnasmOp = match cond_value {
        NodeOutput::Constant(c) => {
            if c.as_bool() {
                // Always true, do nothing and just enter the block
                UnasmOp::Nop
            } else {
                // Always false, jump without examining the condition.
                opcodes::Jump::from(scope.next_instruction()).into()
            }
        }
        _ => opcodes::JumpNot::from((cond_reg, scope.next_instruction())).into(),
    };

    // Now that we know how big our body is, we can update our jump instruction for
    // a false condition to move to this location.
    scope.overwrite(pending_skip_body, jump_op);

    Ok(())
}
