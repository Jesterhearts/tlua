use tlua_bytecode::{
    binop::{
        traits::{
            ComparisonOpEval,
            OpName,
        },
        CompareOp,
    },
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

impl<OpTy> ApplyBinop for CompareOp<OpTy, Register>
where
    OpTy: OpName,
    CompareOp<OpTy, Register>: ComparisonOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        // TODO: metatables
        scopes.store(
            self.lhs,
            Value::Bool(
                match (
                    scopes.load(self.lhs),
                    Value::try_from(self.rhs).unwrap_or_else(|reg| scopes.load(reg)),
                ) {
                    (Value::Nil, Value::Nil) => Self::apply_nils()?,
                    (Value::Bool(lhs), Value::Bool(rhs)) => Self::apply_bools(lhs, rhs)?,
                    (Value::Number(lhs), Value::Number(rhs)) => Self::apply_numbers(lhs, rhs),
                    (Value::String(lhs), Value::String(rhs)) => {
                        Self::apply_strings(&*(*lhs).borrow(), &*(*rhs).borrow())
                    }
                    // TODO: Metatables
                    _ => false,
                },
            ),
        );

        Ok(())
    }
}
