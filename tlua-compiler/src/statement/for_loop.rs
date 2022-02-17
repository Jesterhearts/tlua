use scopeguard::guard_on_success;
use tlua_bytecode::{
    opcodes,
    ImmediateRegister,
    OpError,
    PrimitiveType,
    TypeId,
};
use tlua_parser::ast::statement::for_loop::ForLoop;

use crate::{
    compiler::{
        InitRegister,
        JumpTemplate,
    },
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
        let typecheck0 = scope.push_immediate().no_init_needed();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(typecheck0));

        let typecheck1 = scope.push_immediate().no_init_needed();
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(typecheck1));

        let init = self.init.compile(&mut scope)?;

        let init = emit_assert_isnum(
            &mut scope,
            init,
            typecheck0,
            typecheck1,
            OpError::InvalidForInit,
        );
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(init));

        let limit = self.condition.compile(&mut scope)?;
        let limit = emit_assert_isnum(
            &mut scope,
            limit,
            typecheck0,
            typecheck1,
            OpError::InvalidForCond,
        );
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(limit));

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
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(step));

        let zero = scope.push_immediate().init_from_const(&mut scope, 0.into());
        let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(zero));

        let pending_skip_flip_step = {
            // Check for a negative step, we always want to be dealing with negative steps
            // for simplicity.
            let ge_zero = scope.push_immediate().init_from_immediate(&mut scope, step);
            let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(ge_zero));

            scope.emit(opcodes::GreaterEqual::from((ge_zero, zero)));

            // If the step is negative, skip the extra work to negate it.
            JumpTemplate::<opcodes::JumpNot>::conditional_at(scope.reserve_jump_isn(), ge_zero)
        };

        {
            // Check for a zero step to raise an error.
            let gt_zero = scope.push_immediate().init_from_immediate(&mut scope, step);
            let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(gt_zero));

            scope.emit(opcodes::GreaterThan::from((gt_zero, zero)));

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

        pending_skip_flip_step.resolve_to(scope.next_instruction(), &mut scope);

        let cond_check_start = scope.next_instruction();

        let pending_skip_body = {
            let cond_outcome = scope.push_immediate().init_from_immediate(&mut scope, init);
            let mut scope = guard_on_success(&mut scope, |scope| scope.pop_immediate(cond_outcome));

            scope.emit(opcodes::GreaterEqual::from((cond_outcome, limit)));

            JumpTemplate::<opcodes::JumpNot>::conditional_at(scope.reserve_jump_isn(), cond_outcome)
        };

        scope
            .new_local(self.var)?
            .init_from_immediate(&mut scope, init);
        scope.emit(opcodes::Add::from((init, step)));

        self.body.compile(&mut scope)?;

        scope.emit(opcodes::Jump::from(cond_check_start));

        pending_skip_body.resolve_to(scope.next_instruction(), &mut scope);

        scope.label_current_instruction(loop_exit_label)?;
        scope.pop_loop_label();

        Ok(None)
    }
}

fn emit_assert_isnum(
    scope: &mut Scope,
    target: NodeOutput,
    typecheck0: ImmediateRegister,
    typecheck1: ImmediateRegister,
    err: OpError,
) -> ImmediateRegister {
    let target = match target {
        target @ NodeOutput::Constant(Constant::Integer(_))
        | target @ NodeOutput::Constant(Constant::Float(_)) => {
            return target.into_register(scope);
        }
        NodeOutput::Constant(_) => {
            scope.emit(opcodes::Raise::from(err));
            return scope.push_immediate().no_init_needed();
        }
        target => target.into_register(scope),
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
    scope.emit(opcodes::Or::from((typecheck0, typecheck1)));
    scope.emit(opcodes::RaiseIfNot::from((typecheck0, err)));

    target
}
