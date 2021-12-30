use tlua_bytecode::{
    binop,
    Constant,
};
use tlua_parser::ast::expressions::operator::*;

use crate::{
    compiler::unasm::UnasmRegister,
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for Plus<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::Add<UnasmRegister, Constant>, binop::AddIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Minus<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::Subtract<UnasmRegister, Constant>, binop::SubtractIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Times<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::Times<UnasmRegister, Constant>, binop::TimesIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Divide<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::Divide<UnasmRegister, Constant>, binop::DivideIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for IDiv<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::IDiv<UnasmRegister, Constant>, binop::IDivIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Modulo<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::Modulo<UnasmRegister, Constant>, binop::ModuloIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Exponetiation<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::Exponetiation<UnasmRegister, Constant>, binop::ExponetiationIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for BitAnd<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::BitAnd<UnasmRegister, Constant>, binop::BitAndIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for BitOr<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::BitOr<UnasmRegister, Constant>, binop::BitOrIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for BitXor<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::BitXor<UnasmRegister, Constant>, binop::BitXorIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for ShiftLeft<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::ShiftLeft<UnasmRegister, Constant>, binop::ShiftLeftIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for ShiftRight<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_numeric_binop::<binop::ShiftRight<UnasmRegister, Constant>, binop::ShiftRightIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Concat<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
    }
}

impl CompileExpression for LessThan<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<binop::LessThan<UnasmRegister, Constant>, binop::LessThanIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for LessEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<binop::LessEqual<UnasmRegister, Constant>, binop::LessEqualIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for GreaterThan<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<binop::GreaterThan<UnasmRegister, Constant>, binop::GreaterThanIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for GreaterEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<binop::GreaterEqual<UnasmRegister, Constant>, binop::GreaterEqualIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Equals<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<binop::Equals<UnasmRegister, Constant>, binop::EqualsIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for NotEqual<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_cmp_binop::<binop::NotEqual<UnasmRegister, Constant>, binop::NotEqualIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for And<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_boolean_binop::<binop::And<UnasmRegister, Constant>, binop::AndIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

impl CompileExpression for Or<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_boolean_binop::<binop::Or<UnasmRegister, Constant>, binop::OrIndirect<UnasmRegister, UnasmRegister>, _, _>(self.lhs, self.rhs)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use tlua_bytecode::{
        Constant,
        OpError,
    };
    use tlua_parser::ast::expressions::{
        number::Number,
        operator::*,
        Expression,
        Nil,
    };

    use crate::{
        compiler::Compiler,
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
    // TODO(lang-5.4): Needs different handling than the rest of the cmp ops
    #[ignore]
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
