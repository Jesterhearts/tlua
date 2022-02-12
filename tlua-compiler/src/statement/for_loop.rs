use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    OpError,
    PrimitiveType,
    TypeId,
};
use tlua_parser::ast::statement::for_loop::ForLoop;

use crate::{
    compiler::InitRegister,
    constant::Constant,
    CompileError,
    CompileExpression,
    CompileStatement,
    NodeOutput,
    Scope,
};

impl CompileStatement for ForLoop<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<Option<OpError>, CompileError> {
        let mut scope = scope.new_block();
        let mut scope = scope.enter();

        let loop_exit_label = scope.push_loop_label();
        let typecheck0 = scope.push_anon_reg().no_init_needed();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(typecheck0));

        let typecheck1 = scope.push_anon_reg().no_init_needed();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(typecheck1));

        let init = self.init.compile(&mut scope)?;

        let init = emit_assert_isnum(
            &mut scope,
            init,
            typecheck0,
            typecheck1,
            OpError::InvalidForInit,
        );
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(init));

        let limit = self.condition.compile(&mut scope)?;
        let limit = emit_assert_isnum(
            &mut scope,
            limit,
            typecheck0,
            typecheck1,
            OpError::InvalidForCond,
        );
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(limit));

        let step = self
            .increment
            .as_ref()
            .map(|inc| inc.compile(&mut scope))
            .unwrap_or(Ok(NodeOutput::Constant(Constant::Integer(1))))?;

        let step = emit_assert_isnum(
            &mut scope,
            step,
            typecheck0,
            typecheck1,
            OpError::InvalidForStep,
        );
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(step));

        let zero = scope.push_anon_reg().init_from_const(&mut scope, 0.into());
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(zero));

        // Check for a negative step, we always want to be dealing with negative steps
        // for simplicity.
        let ge_zero = scope.push_anon_reg().no_init_needed();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(ge_zero));

        scope.emit(opcodes::GreaterEqual::from((ge_zero, step, zero)));

        // If the step is negative, skip the extra work to negate it.
        let pending_skip_flip_step = scope.reserve_jump_isn();

        {
            // Check for a zero step to raise an error.
            let gt_zero = scope.push_anon_reg().no_init_needed();
            let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(gt_zero));

            scope.emit(opcodes::GreaterThan::from((gt_zero, step, zero)));

            scope.emit(opcodes::RaiseIfNot::from((
                gt_zero,
                OpError::InvalidForStep,
            )));
        }

        // Positive step, flip it and terminating condition so they're always negative.
        // This is okay because |i64::MIN| >= i64::MAX
        scope.emit(opcodes::UnaryMinus::from((init, init)));
        scope.emit(opcodes::UnaryMinus::from((limit, limit)));
        scope.emit(opcodes::UnaryMinus::from((step, step)));

        {
            let next_isn = scope.next_instruction();
            scope.overwrite(
                pending_skip_flip_step,
                opcodes::JumpNot::from((ge_zero, next_isn)),
            );
        }

        let cond_check_start = scope.next_instruction();
        let cond_outcome = scope.push_anon_reg().no_init_needed();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_anon_reg(cond_outcome));

        scope.emit(opcodes::GreaterEqual::from((cond_outcome, init, limit)));

        let pending_skip_body = scope.reserve_jump_isn();

        scope
            .new_local(self.var)?
            .init_from_anon_reg(&mut scope, init);

        self.body.compile(&mut scope)?;

        scope.emit(opcodes::Add::from((init, init, step)));
        scope.emit(opcodes::Jump::from(cond_check_start));

        {
            let next_isn = scope.next_instruction();
            scope.overwrite(
                pending_skip_body,
                opcodes::JumpNot::from((cond_outcome, next_isn)),
            );
        }

        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}

fn emit_assert_isnum(
    scope: &mut Scope,
    target: NodeOutput,
    typecheck0: AnonymousRegister,
    typecheck1: AnonymousRegister,
    err: OpError,
) -> AnonymousRegister {
    let target = match target {
        target @ NodeOutput::Constant(Constant::Integer(_))
        | target @ NodeOutput::Constant(Constant::Float(_)) => {
            return target.to_register(scope);
        }
        NodeOutput::Constant(_) => {
            scope.emit(opcodes::Raise::from(err));
            return scope.push_anon_reg().no_init_needed();
        }
        target => target.to_register(scope),
    };

    scope.emit(opcodes::CheckType::from((
        typecheck0,
        target,
        TypeId::Primitive(PrimitiveType::Integer),
    )));
    scope.emit(opcodes::CheckType::from((
        typecheck1,
        target,
        TypeId::Primitive(PrimitiveType::Float),
    )));
    scope.emit(opcodes::Or::from((typecheck0, typecheck0, typecheck1)));
    scope.emit(opcodes::RaiseIfNot::from((typecheck0, err)));

    target
}
