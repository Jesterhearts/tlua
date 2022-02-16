pub mod traits;

use tlua_bytecode::{
    opcodes::{
        traits::{
            BooleanOpEval,
            ComparisonOpEval,
            ConcatBinop,
            FloatBinop,
            IntBinop,
            NumericOpEval,
            OpName,
        },
        Concat,
    },
    ImmediateRegister,
    LuaString,
    OpError,
};

use crate::vm::runtime::{
    execution_context::Immediates,
    Value,
};

pub(crate) fn bool_op<Op: BooleanOpEval>(
    lhs: ImmediateRegister,
    rhs: ImmediateRegister,
    registers: &Immediates,
) -> Value {
    Op::evaluate::<&Value, _, _>(&registers[lhs], &registers[rhs]).clone()
}

pub(crate) fn cmp_op<Op: ComparisonOpEval>(
    lhs: ImmediateRegister,
    rhs: ImmediateRegister,
    registers: &Immediates,
) -> Result<Value, OpError> {
    Ok(Value::Bool(match (&registers[lhs], &registers[rhs]) {
        (Value::Nil, Value::Nil) => Op::apply_nils()?,
        (Value::Bool(lhs), Value::Bool(rhs)) => Op::apply_bools(*lhs, *rhs)?,
        (Value::Number(lhs), Value::Number(rhs)) => Op::apply_numbers(*lhs, *rhs),
        (Value::String(lhs), Value::String(rhs)) => {
            Op::apply_strings(&*(*lhs).borrow(), &*(*rhs).borrow())
        }
        _ => false,
    }))
}

pub(crate) fn fp_op<Op: NumericOpEval + FloatBinop + OpName>(
    lhs: ImmediateRegister,
    rhs: ImmediateRegister,
    registers: &Immediates,
) -> Result<Value, OpError> {
    match registers[lhs] {
        Value::Number(n) => Ok(Value::Number(Op::evaluate(&n, &registers[rhs])?)),
        _ => Err(OpError::InvalidType { op: Op::NAME }),
    }
}

pub(crate) fn int_op<Op: NumericOpEval + IntBinop + OpName>(
    lhs: ImmediateRegister,
    rhs: ImmediateRegister,
    registers: &Immediates,
) -> Result<Value, OpError> {
    match registers[lhs] {
        Value::Number(lhs) => Ok(Value::Number(Op::evaluate(&lhs, &registers[rhs])?)),
        _ => Err(OpError::InvalidType { op: Op::NAME }),
    }
}

pub(crate) fn concat_op(
    lhs: ImmediateRegister,
    rhs: ImmediateRegister,
    registers: &Immediates,
) -> Result<Value, OpError> {
    match (&registers[lhs], &registers[rhs]) {
        (Value::Number(lhs), Value::Number(rhs)) => {
            Ok(Concat::evaluate(LuaString::from(lhs), LuaString::from(rhs)))
        }
        (Value::Number(lhs), Value::String(rhs)) => {
            Ok(Concat::evaluate(LuaString::from(lhs), &*rhs.borrow()))
        }
        (Value::String(lhs), Value::Number(rhs)) => {
            Ok(Concat::evaluate(&*lhs.borrow(), LuaString::from(rhs)))
        }
        (Value::String(lhs), Value::String(rhs)) => {
            Ok(Concat::evaluate(&*lhs.borrow(), &*rhs.borrow()))
        }
        _ => Err(OpError::InvalidType { op: Concat::NAME }),
    }
}
