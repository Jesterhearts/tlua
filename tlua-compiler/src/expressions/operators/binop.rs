use tlua_bytecode::{
    self,
    binop::{
        traits::{
            BooleanOpEval,
            ComparisonOpEval,
            NumericOpEval,
        },
        *,
    },
    OpError,
};
use tlua_parser::ast::expressions::{
    operator,
    Expression,
};

use crate::{
    compiler::unasm::{
        LocalRegister,
        UnasmOp,
        UnasmOperand,
        UnasmRegister,
    },
    constant::Constant,
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

fn write_numeric_binop<Op>(
    compiler: &mut CompilerContext,
    lhs: &Expression,
    rhs: &Expression,
) -> Result<NodeOutput, CompileError>
where
    Op: NumericOpEval + From<(UnasmRegister, UnasmOperand)> + Into<UnasmOp>,
{
    compiler.write_binop::<Op, _, _, _>(lhs, rhs, |lhs, rhs| {
        Op::evaluate(lhs, rhs).map(|num| num.into())
    })
}

fn write_cmp_binop<Op>(
    compiler: &mut CompilerContext,
    lhs: &Expression,
    rhs: &Expression,
) -> Result<NodeOutput, CompileError>
where
    Op: ComparisonOpEval + From<(UnasmRegister, UnasmOperand)> + Into<UnasmOp>,
{
    compiler.write_binop::<Op, _, _, _>(lhs, rhs, |lhs, rhs| match (lhs, rhs) {
        (Constant::Nil, Constant::Nil) => Op::apply_nils().map(Constant::from),
        (Constant::Bool(lhs), Constant::Bool(rhs)) => Op::apply_bools(lhs, rhs).map(Constant::from),
        (Constant::Float(lhs), Constant::Float(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::Float(lhs), Constant::Integer(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::Integer(lhs), Constant::Integer(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::Integer(lhs), Constant::Float(rhs)) => {
            Ok(Op::apply_numbers(lhs.into(), rhs.into()).into())
        }
        (Constant::String(lhs), Constant::String(rhs)) => Ok(Op::apply_strings(&lhs, &rhs).into()),
        // TODO(lang-5.4): This should be truthy for eq/ne.
        (lhs, rhs) => Err(OpError::CmpErr {
            lhs: lhs.short_type_name(),
            rhs: rhs.short_type_name(),
        }),
    })
}

fn write_boolean_binop<Op>(
    compiler: &mut CompilerContext,
    lhs: &Expression,
    rhs: &Expression,
) -> Result<NodeOutput, CompileError>
where
    Op: BooleanOpEval + From<(UnasmRegister, UnasmOperand)> + Into<UnasmOp>,
{
    compiler.write_binop::<Op, _, _, _>(lhs, rhs, |lhs, rhs| Ok(Op::evaluate(lhs, rhs)))
}

impl CompileExpression for operator::Plus<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<Add, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Minus<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<Subtract, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Times<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<Times, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Divide<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<Divide, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::IDiv<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<IDiv, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Modulo<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<Modulo, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Exponetiation<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<FloatOp<Exponetiation, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::BitAnd<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<IntOp<BitAnd, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::BitOr<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<IntOp<BitOr, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::BitXor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<IntOp<BitXor, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::ShiftLeft<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<IntOp<ShiftLeft, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::ShiftRight<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_numeric_binop::<IntOp<ShiftRight, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Concat<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}

impl CompileExpression for operator::LessThan<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<CompareOp<LessThan, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::LessEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<CompareOp<LessEqual, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::GreaterThan<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<CompareOp<GreaterThan, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::GreaterEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<CompareOp<GreaterEqual, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Equals<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<CompareOp<Equals, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::NotEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_cmp_binop::<CompareOp<NotEqual, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::And<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_boolean_binop::<BoolOp<And, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

impl CompileExpression for operator::Or<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        write_boolean_binop::<BoolOp<Or, LocalRegister>>(compiler, self.lhs, self.rhs)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tlua_bytecode::OpError;
    use tlua_parser::ast::expressions::{
        number::Number,
        operator::*,
        Expression,
        Nil,
    };

    use crate::{
        compiler::Compiler,
        constant::Constant,
        CompileExpression,
        NodeOutput,
    };

    #[test]
    fn compiles_constant_plus() -> anyhow::Result<()> {
        let ast = Plus {
            lhs: &Expression::Number(Number::Integer(1)),
            rhs: &Expression::Number(Number::Integer(2)),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Integer(3)));

        Ok(())
    }

    #[test]
    fn compiles_constant_eq() -> anyhow::Result<()> {
        let ast = Equals {
            lhs: &Expression::Number(Number::Float(4.0)),
            rhs: &Expression::Number(Number::Integer(4)),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(true)));

        Ok(())
    }

    #[test]
    fn compiles_constant_eq_false() -> anyhow::Result<()> {
        let ast = Equals {
            lhs: &Expression::Number(Number::Integer(1)),
            rhs: &Expression::Number(Number::Integer(2)),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(false)));

        Ok(())
    }

    #[test]
    #[ignore = "TODO(lang-5.4): Needs different handling than the rest of the cmp ops"]
    fn compiles_constant_eq_types_dif_false() -> anyhow::Result<()> {
        let ast = Equals {
            lhs: &Expression::Number(Number::Integer(1)),
            rhs: &Expression::Bool(true),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(false)));

        Ok(())
    }

    #[test]
    fn compiles_constant_eq_strings() -> anyhow::Result<()> {
        let ast = Equals {
            lhs: &Expression::String("test".into()),
            rhs: &Expression::String("test".into()),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(true)));

        Ok(())
    }

    #[test]
    fn compiles_constant_eq_nil() -> anyhow::Result<()> {
        let ast = Equals {
            lhs: &Expression::Nil(Nil),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(true)));

        Ok(())
    }

    #[test]
    fn compiles_constant_lt_nums() -> anyhow::Result<()> {
        let ast = LessThan {
            lhs: &Expression::Number(Number::Integer(10)),
            rhs: &Expression::Number(Number::Float(11.0)),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(true)));

        Ok(())
    }

    #[test]
    fn compiles_constant_lt_nums_false() -> anyhow::Result<()> {
        let ast = LessThan {
            lhs: &Expression::Number(Number::Integer(11)),
            rhs: &Expression::Number(Number::Integer(10)),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(false)));

        Ok(())
    }

    #[test]
    fn compiles_constant_lt_strings() -> anyhow::Result<()> {
        let ast = LessThan {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::String("def".into()),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Bool(true)));

        Ok(())
    }

    #[test]
    fn compiles_constant_lt_mixed() -> anyhow::Result<()> {
        let ast = LessThan {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(
            result,
            NodeOutput::Err(OpError::CmpErr {
                lhs: Constant::String("abc".into()).short_type_name(),
                rhs: Constant::Nil.short_type_name()
            })
        );

        Ok(())
    }

    #[test]
    fn and_truthy() -> anyhow::Result<()> {
        let ast = And {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::Number(Number::Integer(10)),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Integer(10)));

        Ok(())
    }

    #[test]
    fn and_truthy_falsy() -> anyhow::Result<()> {
        let ast = And {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Nil));

        Ok(())
    }

    #[test]
    fn or_truthy_falsy() -> anyhow::Result<()> {
        let ast = Or {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::String("abc".into())));

        Ok(())
    }

    #[test]
    fn or_falsy() -> anyhow::Result<()> {
        let ast = Or {
            lhs: &Expression::Bool(false),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Nil));

        Ok(())
    }
}
