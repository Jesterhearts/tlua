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
    CompilerContext,
    NodeOutput,
};

impl CompileStatement for ForLoop<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<Option<OpError>, CompileError> {
        compiler.emit_in_subscope(|compiler| {
            let loop_exit_label = compiler.push_loop_label();
            let typecheck0 = compiler.new_anon_reg().no_init_needed();
            let typecheck1 = compiler.new_anon_reg().no_init_needed();

            let init = self.init.compile(compiler)?;

            let init = emit_assert_isnum(
                compiler,
                init,
                typecheck0,
                typecheck1,
                OpError::InvalidForInit,
            );

            let limit = self.condition.compile(compiler)?;
            let limit = emit_assert_isnum(
                compiler,
                limit,
                typecheck0,
                typecheck1,
                OpError::InvalidForCond,
            );

            let step = self
                .increment
                .as_ref()
                .map(|inc| inc.compile(compiler))
                .unwrap_or(Ok(NodeOutput::Constant(Constant::Integer(1))))?;

            let step = emit_assert_isnum(
                compiler,
                step,
                typecheck0,
                typecheck1,
                OpError::InvalidForStep,
            );

            let zero = compiler.new_anon_reg().init_from_const(compiler, 0.into());

            // Check for a negative step, we always want to be dealing with negative steps
            // for simplicity.
            let ge_zero = compiler.new_anon_reg().no_init_needed();
            compiler.emit(opcodes::GreaterEqual::from((ge_zero, step, zero)));

            // If the step is negative, skip the extra work to negate it.
            let pending_skip_flip_step = compiler.emit(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: compiler.next_instruction(),
                },
            });

            // Check for a zero step to raise an error.
            let gt_zero = compiler.new_anon_reg().no_init_needed();
            compiler.emit(opcodes::GreaterThan::from((gt_zero, step, zero)));

            compiler.emit(opcodes::RaiseIfNot::from((
                gt_zero,
                OpError::InvalidForStep,
            )));

            // Positive step, flip it and terminating condition so they're always negative.
            // This is okay because |i64::MIN| >= i64::MAX
            compiler.emit(opcodes::UnaryMinus::from((init, init)));
            compiler.emit(opcodes::UnaryMinus::from((limit, limit)));
            compiler.emit(opcodes::UnaryMinus::from((step, step)));

            compiler.overwrite(
                pending_skip_flip_step,
                opcodes::JumpNot::from((ge_zero, compiler.next_instruction())),
            );

            let cond_check_start = compiler.next_instruction();
            let cond_outcome = compiler.new_anon_reg().no_init_needed();
            compiler.emit(opcodes::GreaterEqual::from((cond_outcome, init, limit)));

            let pending_skip_body = compiler.emit(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: compiler.next_instruction(),
                },
            });

            compiler
                .new_local(self.var)?
                .init_from_anon_reg(compiler, init);

            self.body.compile(compiler)?;

            compiler.emit(opcodes::Add::from((init, init, step)));
            compiler.emit(opcodes::Jump::from(cond_check_start));

            compiler.overwrite(
                pending_skip_body,
                opcodes::JumpNot::from((cond_outcome, compiler.next_instruction())),
            );

            compiler.label_current_instruction(loop_exit_label)?;
            compiler.pop_loop_label();

            Ok(None)
        })
    }
}

fn emit_assert_isnum(
    compiler: &mut CompilerContext,
    target: NodeOutput,
    typecheck0: AnonymousRegister,
    typecheck1: AnonymousRegister,
    err: OpError,
) -> AnonymousRegister {
    let target = match target {
        target @ NodeOutput::Constant(Constant::Integer(_))
        | target @ NodeOutput::Constant(Constant::Float(_)) => {
            return compiler
                .new_anon_reg()
                .init_from_node_output(compiler, target);
        }
        NodeOutput::Constant(_) => {
            compiler.emit(opcodes::Raise::from(err));
            return compiler.new_anon_reg().no_init_needed();
        }
        target => compiler
            .new_anon_reg()
            .init_from_node_output(compiler, target),
    };

    compiler.emit(opcodes::CheckType::from((
        typecheck0,
        target,
        TypeId::Primitive(PrimitiveType::Integer),
    )));
    compiler.emit(opcodes::CheckType::from((
        typecheck1,
        target,
        TypeId::Primitive(PrimitiveType::Float),
    )));
    compiler.emit(opcodes::Or::from((typecheck0, typecheck0, typecheck1)));
    compiler.emit(opcodes::RaiseIfNot::from((typecheck0, err)));

    target
}
