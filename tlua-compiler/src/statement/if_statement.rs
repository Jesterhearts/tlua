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
        JumpTemplate,
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
    let jump_template = match cond_value {
        NodeOutput::Constant(c) => {
            if c.as_bool() {
                // Always true, do nothing and just enter the block
                None
            } else {
                // Always false, jump without examining the condition.
                Some(JumpTemplate::unconditional_at(scope.reserve_jump_isn()))
            }
        }
        cond_value => {
            let reg = cond_value.into_register(scope);
            let mut scope = guard_on_success(&mut *scope, |scope| scope.pop_immediate(reg));
            Some(JumpTemplate::<opcodes::JumpNot>::conditional_at(
                scope.reserve_jump_isn(),
                reg,
            ))
        }
    };

    body.compile(scope)?;

    scope.emit_jump_label(exit_label);
    if let Some(jump) = jump_template {
        jump.resolve_to(scope.next_instruction(), scope);
    }

    Ok(())
}
