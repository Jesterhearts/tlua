use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::{
        UnaryBitNot,
        UnaryMinus,
    },
    Truthy,
};
use tlua_parser::ast::expressions::operator::*;

use crate::{
    compiler::unasm::LocalRegister,
    constant::Constant,
    CompileError,
    CompileExpression,
    CompilerContext,
    NodeOutput,
};

impl CompileExpression for Negation<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_unary_op::<UnaryMinus<LocalRegister>, _, _>(&self.0, |v| match v {
            Constant::Float(f) => Ok((-f).into()),
            Constant::Integer(i) => Ok((-i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "negation" }),
        })
    }
}

impl CompileExpression for Not<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_unary_op::<UnaryMinus<LocalRegister>, _, _>(&self.0, |v| {
            Ok((!v.as_bool()).into())
        })
    }
}

impl CompileExpression for BitNot<'_> {
    fn compile(&self, compiler: &mut CompilerContext) -> Result<NodeOutput, CompileError> {
        compiler.write_unary_op::<UnaryBitNot<LocalRegister>, _, _>(&self.0, |v| match v {
            Constant::Float(f) => f64inbounds(f).map(|i| (!i).into()),
            Constant::Integer(i) => Ok((!i).into()),
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
    use tlua_bytecode::OpError;
    use tlua_parser::ast::expressions::{
        number::Number,
        operator::BitNot,
        Expression,
    };

    use crate::{
        compiler::Compiler,
        constant::Constant,
        CompileExpression,
        NodeOutput,
    };

    #[test]
    fn exact_dec_f64_bitnot() -> anyhow::Result<()> {
        let ast = BitNot(&Expression::Number(Number::Float(10.0)));

        let mut compiler = Compiler::default();
        compiler.emit_in_main(|context| {
            let result = ast.compile(context)?;

            assert_eq!(result, NodeOutput::Constant(Constant::Integer(-11)));
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn i64_bitnot() -> anyhow::Result<()> {
        let ast = BitNot(&Expression::Number(Number::Integer(0)));

        let mut compiler = Compiler::default();
        compiler.emit_in_main(|context| {
            let result = ast.compile(context)?;

            assert_eq!(result, NodeOutput::Constant(Constant::Integer(-1)));
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn string_bitnot() -> anyhow::Result<()> {
        let mut compiler = Compiler::default();
        compiler.emit_in_main(|context| {
            let result = BitNot(&Expression::String("abc".into())).compile(context)?;

            assert_eq!(
                result,
                NodeOutput::Err(OpError::InvalidType { op: "bitwise" })
            );

            Ok(())
        })?;

        Ok(())
    }
}
