use tlua_bytecode::{
    binop::f64inbounds,
    opcodes::{
        self,
        UnaryBitNot,
        UnaryMinus,
    },
    Constant,
    ImmediateRegister,
    OpError,
    Truthy,
};
use tlua_parser::{
    expressions::operator::*,
    StringTable,
};

use crate::{
    compiler::unasm::UnasmOp,
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
    Op: From<(ImmediateRegister, ImmediateRegister)> + Into<UnasmOp>,
    Operand: CompileExpression,
    ConstEval: FnOnce(&StringTable, Constant) -> Result<Constant, OpError>,
{
    match operand.compile(scope)? {
        NodeOutput::Constant(c) => match consteval(scope.string_table(), c) {
            Ok(val) => Ok(NodeOutput::Constant(val)),
            Err(err) => Ok(NodeOutput::Err(scope.write_raise(err))),
        },
        src => {
            let src = src.into_register(scope);
            scope.emit(Op::from((src, src)));

            Ok(NodeOutput::Immediate(src))
        }
    }
}

impl CompileExpression for Negation<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_unary_op::<UnaryMinus, _, _>(scope, &self.0, |_, v| match v {
            Constant::Float(f) => Ok((-f).into()),
            Constant::Integer(i) => Ok((-i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "negation" }),
        })
    }
}

impl CompileExpression for BitNot<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_unary_op::<UnaryBitNot, _, _>(scope, &self.0, |_, v| match v {
            Constant::Float(f) => f64inbounds(f).map(|i| (!i).into()),
            Constant::Integer(i) => Ok((!i).into()),
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "bitwise not" }),
        })
    }
}

impl CompileExpression for Not<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_unary_op::<opcodes::Not, _, _>(scope, &self.0, |_, v| Ok((!v.as_bool()).into()))
    }
}

impl CompileExpression for Length<'_> {
    fn compile(&self, scope: &mut Scope) -> Result<NodeOutput, CompileError> {
        write_unary_op::<opcodes::Length, _, _>(scope, &self.0, |strings, v| match v {
            Constant::String(s) => {
                let s = strings.get_string(s).expect("Valid string id");
                i64::try_from(s.len())
                    .map(Constant::from)
                    .map_err(|_| tlua_bytecode::OpError::StringLengthOutOfBounds)
            }
            _ => Err(tlua_bytecode::OpError::InvalidType { op: "length" }),
        })
    }
}
