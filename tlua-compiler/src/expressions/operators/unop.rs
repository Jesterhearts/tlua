use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::UnaryMinus,
    OpError,
    Truthy,
};
use tlua_parser::ast::expressions::operator::*;

use crate::{
    compiler::unasm::UnasmRegister,
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for Negation<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_unary_op::<UnaryMinus<UnasmRegister>, _, _>(&self.0, |v| match v {
            tlua_bytecode::Constant::Float(f) => Ok((-f).into()),
            tlua_bytecode::Constant::Integer(i) => Ok((-i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "negation" }),
        })
    }
}

impl CompileExpression for Not<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_unary_op::<UnaryMinus<UnasmRegister>, _, _>(&self.0, |v| {
            Ok((!v.as_bool()).into())
        })
    }
}

impl CompileExpression for BitNot<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_unary_op::<UnaryMinus<UnasmRegister>, _, _>(&self.0, |v| match v {
            tlua_bytecode::Constant::Float(f) => {
                if f.fract() == 0.0 {
                    f64inbounds(f).map(|i| (!i).into())
                } else {
                    Err(OpError::FloatToIntConversionFailed { f })
                }
            }
            tlua_bytecode::Constant::Integer(i) => Ok((!i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "bitwise" }),
        })
    }
}

impl CompileExpression for Length<'_> {
    fn compile(&self, _compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        todo!()
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
        operator::BitNot,
        Expression,
    };

    use crate::{
        compiler::Compiler,
        CompileExpression,
        NodeOutput,
    };

    #[test]
    fn exact_dec_f64_bitnot() -> anyhow::Result<()> {
        let ast = BitNot(&Expression::Number(Number::Float(10.0)));

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Integer(-11)));

        Ok(())
    }

    #[test]
    fn i64_bitnot() -> anyhow::Result<()> {
        let ast = BitNot(&Expression::Number(Number::Integer(0)));

        let mut compiler = Compiler::default();
        let result = ast.compile(&mut compiler.new_context())?;

        assert_eq!(result, NodeOutput::Constant(Constant::Integer(-1)));

        Ok(())
    }

    #[test]
    fn string_bitnot() -> anyhow::Result<()> {
        let mut compiler = Compiler::default();
        let result =
            BitNot(&Expression::String("abc".into())).compile(&mut compiler.new_context())?;

        assert_eq!(
            result,
            NodeOutput::Err(OpError::InvalidType { op: "bitwise" })
        );

        Ok(())
    }
}
