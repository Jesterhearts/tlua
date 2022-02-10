use tlua_bytecode::{
    opcodes,
    AnonymousRegister,
    ByteCodeError,
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
        let typecheck0 = scope.new_anon_reg().no_init_needed();
        let typecheck1 = scope.new_anon_reg().no_init_needed();

        let init = self.init.compile(&mut scope)?;

        let init = emit_assert_isnum(
            &mut scope,
            init,
            typecheck0,
            typecheck1,
            OpError::InvalidForInit,
        );

        let limit = self.condition.compile(&mut scope)?;
        let limit = emit_assert_isnum(
            &mut scope,
            limit,
            typecheck0,
            typecheck1,
            OpError::InvalidForCond,
        );

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

        let zero = scope.new_anon_reg().init_from_const(&mut scope, 0.into());

        // Check for a negative step, we always want to be dealing with negative steps
        // for simplicity.
        let ge_zero = scope.new_anon_reg().no_init_needed();
        scope.emit(opcodes::GreaterEqual::from((ge_zero, step, zero)));

        // If the step is negative, skip the extra work to negate it.
        let pending_skip_flip_step = scope.emit(opcodes::Raise {
            err: OpError::ByteCodeError {
                err: ByteCodeError::MissingJump,
                offset: scope.next_instruction(),
            },
        });

        // Check for a zero step to raise an error.
        let gt_zero = scope.new_anon_reg().no_init_needed();
        scope.emit(opcodes::GreaterThan::from((gt_zero, step, zero)));

        scope.emit(opcodes::RaiseIfNot::from((
            gt_zero,
            OpError::InvalidForStep,
        )));

        // Positive step, flip it and terminating condition so they're always negative.
        // This is okay because |i64::MIN| >= i64::MAX
        scope.emit(opcodes::UnaryMinus::from((init, init)));
        scope.emit(opcodes::UnaryMinus::from((limit, limit)));
        scope.emit(opcodes::UnaryMinus::from((step, step)));

        scope.overwrite(
            pending_skip_flip_step,
            opcodes::JumpNot::from((ge_zero, scope.next_instruction())),
        );

        let cond_check_start = scope.next_instruction();
        let cond_outcome = scope.new_anon_reg().no_init_needed();
        scope.emit(opcodes::GreaterEqual::from((cond_outcome, init, limit)));

        let pending_skip_body = scope.emit(opcodes::Raise {
            err: OpError::ByteCodeError {
                err: ByteCodeError::MissingJump,
                offset: scope.next_instruction(),
            },
        });

        scope
            .new_local(self.var)?
            .init_from_anon_reg(&mut scope, init);

        self.body.compile(&mut scope)?;

        scope.emit(opcodes::Add::from((init, init, step)));
        scope.emit(opcodes::Jump::from(cond_check_start));

        scope.overwrite(
            pending_skip_body,
            opcodes::JumpNot::from((cond_outcome, scope.next_instruction())),
        );

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
            return scope.new_anon_reg().init_from_node_output(scope, target);
        }
        NodeOutput::Constant(_) => {
            scope.emit(opcodes::Raise::from(err));
            return scope.new_anon_reg().no_init_needed();
        }
        target => scope.new_anon_reg().init_from_node_output(scope, target),
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
