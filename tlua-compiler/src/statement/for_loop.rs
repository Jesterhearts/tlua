use tlua_bytecode::{
    binop,
    opcodes,
    ByteCodeError,
    OpError,
    PrimitiveType,
    TypeId,
};
use tlua_parser::ast::statement::for_loop::ForLoop;

use crate::{
    compiler::{
        unasm::{
            UnasmOperand,
            UnasmRegister,
        },
        InitRegister,
    },
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
            let typecheck0: UnasmRegister = compiler.new_anon_reg().no_init_needed().into();
            let typecheck1: UnasmRegister = compiler.new_anon_reg().no_init_needed().into();

            let init = self.init.compile(compiler)?;
            let init = emit_assert_isnum(
                compiler,
                init,
                |compiler, target| {
                    Ok(compiler
                        // TODO(lang-5.4): This cannot be assigned to.
                        .new_local(self.var)?
                        .init_from_node_output(compiler, target)
                        .into())
                },
                typecheck0,
                typecheck1,
                OpError::InvalidForInit,
            )?;

            let limit = self.condition.compile(compiler)?;
            let limit = emit_assert_isnum(
                compiler,
                limit,
                |compiler, target| {
                    Ok(compiler
                        .new_anon_reg()
                        .init_from_node_output(compiler, target)
                        .into())
                },
                typecheck0,
                typecheck1,
                OpError::InvalidForCond,
            )?;

            let step = self
                .increment
                .as_ref()
                .map(|inc| inc.compile(compiler))
                .unwrap_or(Ok(NodeOutput::Constant(Constant::Integer(1))))?;
            let step = emit_assert_isnum(
                compiler,
                step,
                |compiler, target| {
                    Ok(compiler
                        .new_anon_reg()
                        .init_from_node_output(compiler, target)
                        .into())
                },
                typecheck0,
                typecheck1,
                OpError::InvalidForStep,
            )?;

            // Check for a negative step, we always want to be dealing with negative steps
            // for simplicity.
            let ge_zero: UnasmRegister =
                compiler.new_anon_reg().init_from_reg(compiler, step).into();
            compiler.emit(binop::CompareOp::<binop::GreaterEqual, _>::from((
                ge_zero,
                UnasmOperand::from(0),
            )));

            // If the step is negative, skip the extra work to negate it.
            let pending_skip_flip_step = compiler.emit(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: compiler.next_instruction(),
                },
            });

            // Check for a zero step to raise an error.
            let gt_zero: UnasmRegister =
                compiler.new_anon_reg().init_from_reg(compiler, step).into();
            compiler.emit(binop::CompareOp::<binop::GreaterThan, _>::from((
                gt_zero,
                UnasmOperand::from(0),
            )));

            compiler.emit(opcodes::RaiseIfNot::from((
                gt_zero,
                OpError::InvalidForStep,
            )));

            // Positive step, flip it and terminating condition so they're always negative.
            // This is okay because |i64::MIN| >= i64::MAX
            compiler.emit(opcodes::UnaryMinus::from(init));
            compiler.emit(opcodes::UnaryMinus::from(limit));
            compiler.emit(opcodes::UnaryMinus::from(step));

            compiler.overwrite(
                pending_skip_flip_step,
                opcodes::JumpNot::from((ge_zero, compiler.next_instruction())),
            );

            let cond_check_start = compiler.next_instruction();
            // Copy the cond register value into the outcome register so we can test it.
            let cond_outcome: UnasmRegister =
                compiler.new_anon_reg().init_from_reg(compiler, init).into();
            compiler.emit(binop::CompareOp::<binop::GreaterEqual, _>::from((
                cond_outcome,
                UnasmOperand::from(limit),
            )));

            let pending_skip_body = compiler.emit(opcodes::Raise {
                err: OpError::ByteCodeError {
                    err: ByteCodeError::MissingJump,
                    offset: compiler.next_instruction(),
                },
            });

            self.body.compile(compiler)?;

            compiler.emit(binop::FloatOp::<binop::Add, _>::from((
                init,
                UnasmOperand::from(step),
            )));
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
    mut target_to_reg: impl FnMut(
        &mut CompilerContext,
        NodeOutput,
    ) -> Result<UnasmRegister, CompileError>,
    typecheck0: UnasmRegister,
    typecheck1: UnasmRegister,
    err: OpError,
) -> Result<UnasmRegister, CompileError> {
    let target = match target {
        target @ NodeOutput::Constant(Constant::Integer(_))
        | target @ NodeOutput::Constant(Constant::Float(_)) => {
            return target_to_reg(compiler, target);
        }
        NodeOutput::Constant(_) => {
            compiler.emit(opcodes::Raise::from(err));
            return target_to_reg(compiler, target);
        }
        target => target_to_reg(compiler, target)?,
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
    compiler.emit(binop::BoolOp::<binop::Or, _>::from((
        typecheck0,
        UnasmOperand::from(typecheck1),
    )));
    compiler.emit(opcodes::RaiseIfNot::from((typecheck0, err)));

    Ok(target)
}
