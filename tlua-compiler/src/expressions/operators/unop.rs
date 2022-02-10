use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::{
        self,
        UnaryBitNot,
        UnaryMinus,
    },
    AnonymousRegister,
    OpError,
    Truthy,
};
use tlua_parser::ast::expressions::operator::*;

use crate::{
    compiler::{
        unasm::UnasmOp,
        InitRegister,
    },
    constant::Constant,
    CompileError,
    CompileExpression,
    NodeOutput,
    Scope,
};

pub(crate) fn write_unary_op<Op, Operand, ConstEval>(
    scope: &mut Scope,
    operand: Operand,
    consteval: ConstEval,
) -> Result<NodeOutput, CompileError>
where
    Op: From<(AnonymousRegister, AnonymousRegister)> + Into<UnasmOp>,
    Operand: CompileExpression,
    ConstEval: FnOnce(Constant) -> Result<Constant, OpError>,
{
    match operand.compile(scope)? {
        NodeOutput::Constant(c) => match consteval(c) {
            Ok(val) => Ok(NodeOutput::Constant(val)),
            Err(err) => Ok(NodeOutput::Err(scope.write_raise(err))),
        },
        src => {
            let src = scope.new_anon_reg().init_from_node_output(scope, src);
            let dst = scope.new_anon_reg().no_init_needed();

            scope.emit(Op::from((dst, src)));

            Ok(NodeOutput::Immediate(dst))
        }
    }
}

impl CompileExpression for Negation<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_unary_op::<UnaryMinus, _, _>(scope, &self.0, |v| match v {
            Constant::Float(f) => Ok((-f).into()),
            Constant::Integer(i) => Ok((-i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "negation" }),
        })
    }
}

impl CompileExpression for BitNot<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_unary_op::<UnaryBitNot, _, _>(scope, &self.0, |v| match v {
            Constant::Float(f) => f64inbounds(f).map(|i| (!i).into()),
            Constant::Integer(i) => Ok((!i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "bitwise not" }),
        })
    }
}

impl CompileExpression for Not<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        scope.write_unary_op::<opcodes::Not, _, _>(&self.0, |v| Ok((!v.as_bool()).into()))
    }
}

impl CompileExpression for Length<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        scope.write_unary_op::<opcodes::Length, _, _>(&self.0, |v| match v {
            Constant::String(s) => i64::try_from(s.len())
                .map(Constant::from)
                .map_err(|_| tlua_bytecode::OpError::StringLengthOutOfBounds),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "length" }),
        })
    }
}
