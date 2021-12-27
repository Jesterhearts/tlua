use crate::{
    ast::expressions::operator::*,
    compiling::{
        compiler::unasm::UnasmRegister,
        CompileError,
        CompileExpression,
        CompilerContext,
        NodeOutput,
    },
    vm::{
        self,
        Constant,
    },
};

impl CompileExpression for Plus<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::Add<UnasmRegister, Constant>, vm::binop::AddIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Minus<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::Subtract<UnasmRegister, Constant>, vm::binop::SubtractIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Times<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::Times<UnasmRegister, Constant>, vm::binop::TimesIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Divide<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::Divide<UnasmRegister, Constant>, vm::binop::DivideIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for IDiv<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::IDiv<UnasmRegister, Constant>, vm::binop::IDivIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Modulo<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::Modulo<UnasmRegister, Constant>, vm::binop::ModuloIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Exponetiation<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::Exponetiation<UnasmRegister, Constant>, vm::binop::ExponetiationIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for BitAnd<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::BitAnd<UnasmRegister, Constant>, vm::binop::BitAndIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for BitOr<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::BitOr<UnasmRegister, Constant>, vm::binop::BitOrIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for BitXor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::BitXor<UnasmRegister, Constant>, vm::binop::BitXorIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for ShiftLeft<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::ShiftLeft<UnasmRegister, Constant>, vm::binop::ShiftLeftIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for ShiftRight<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<vm::binop::ShiftRight<UnasmRegister, Constant>, vm::binop::ShiftRightIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Concat<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}

impl CompileExpression for LessThan<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<vm::binop::LessThan<UnasmRegister, Constant>, vm::binop::LessThanIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for LessEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<vm::binop::LessEqual<UnasmRegister, Constant>, vm::binop::LessEqualIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for GreaterThan<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<vm::binop::GreaterThan<UnasmRegister, Constant>, vm::binop::GreaterThanIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for GreaterEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<vm::binop::GreaterEqual<UnasmRegister, Constant>, vm::binop::GreaterEqualIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Equals<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<vm::binop::Equals<UnasmRegister, Constant>, vm::binop::EqualsIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for NotEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<vm::binop::NotEqual<UnasmRegister, Constant>, vm::binop::NotEqualIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for And<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_boolean_binop::<vm::binop::And<UnasmRegister, Constant>, vm::binop::AndIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Or<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_boolean_binop::<vm::binop::Or<UnasmRegister, Constant>, vm::binop::OrIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        ast::expressions::{
            operator::*,
            Expression,
        },
        compiling::{
            compiler::Compiler,
            CompileExpression,
            NodeOutput,
        },
        values::Nil,
        vm::{
            Constant,
            Number,
            OpError,
        },
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
            rhs: &Expression::Number(10.into()),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(true.into()));

        Ok(())
    }

    #[test]
    fn and_falsy() -> anyhow::Result<()> {
        let ast = And {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(false.into()));

        Ok(())
    }

    #[test]
    fn and_truthy_falsy() -> anyhow::Result<()> {
        let ast = Or {
            lhs: &Expression::String("abc".into()),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(true.into()));

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

        assert_eq!(result, NodeOutput::Constant(true.into()));

        Ok(())
    }

    #[test]
    fn or_falsy() -> anyhow::Result<()> {
        let ast = And {
            lhs: &Expression::Bool(false),
            rhs: &Expression::Nil(Nil),
        };

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(false.into()));

        Ok(())
    }
}
