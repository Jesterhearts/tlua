use tlua_bytecode::{
    binop::{
        traits::{
            ComparisonOpEval,
            OpName,
        },
        CompareOp,
    },
    Constant,
    OpError,
    Register,
};

use crate::vm::{
    binop::traits::ApplyBinop,
    runtime::{
        value::function::ScopeSet,
        Value,
    },
};

impl<OpTy> ApplyBinop for CompareOp<OpTy, Register, Constant>
where
    OpTy: OpName,
    CompareOp<OpTy, Register, Constant>: ComparisonOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        // TODO: metatables
        scopes.store(
            self.lhs,
            Value::Bool(match (scopes.load(self.lhs), self.rhs) {
                (Value::Nil, Constant::Nil) => Self::apply_nils()?,
                (Value::Bool(lhs), Constant::Bool(rhs)) => Self::apply_bools(lhs, rhs)?,
                (Value::Number(lhs), Constant::Float(rhs)) => Self::apply_numbers(lhs, rhs.into()),
                (Value::Number(lhs), Constant::Integer(rhs)) => {
                    Self::apply_numbers(lhs, rhs.into())
                }
                (Value::String(lhs), Constant::String(rhs)) => {
                    Self::apply_strings(&*(*lhs).borrow(), &rhs)
                }
                // TODO: Metatables
                _ => false,
            }),
        );

        Ok(())
    }
}

impl<OpTy> ApplyBinop for CompareOp<OpTy, Register, Register>
where
    OpTy: OpName,
    CompareOp<OpTy, Register, Register>: ComparisonOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        // TODO: metatables
        scopes.store(
            self.lhs,
            Value::Bool(match (scopes.load(self.lhs), scopes.load(self.rhs)) {
                (Value::Nil, Value::Nil) => Self::apply_nils()?,
                (Value::Bool(lhs), Value::Bool(rhs)) => Self::apply_bools(lhs, rhs)?,
                (Value::Number(lhs), Value::Number(rhs)) => Self::apply_numbers(lhs, rhs),
                (Value::String(lhs), Value::String(rhs)) => {
                    Self::apply_strings(&*(*lhs).borrow(), &*(*rhs).borrow())
                }
                // TODO: Metatables
                _ => false,
            }),
        );

        Ok(())
    }
}
