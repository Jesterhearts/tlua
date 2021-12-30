use tlua_bytecode::{
    binop::{
        traits::{
            ComparisonOpEval,
            OpName,
        },
        BinOpData,
        CompareOpTag,
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

impl<OpTy> ApplyBinop for BinOpData<CompareOpTag<OpTy>, Register, Constant>
where
    OpTy: OpName + ComparisonOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        // TODO: metatables
        scopes.store(
            self.lhs,
            Value::Bool(match (scopes.load(self.lhs), self.rhs) {
                (Value::Nil, Constant::Nil) => OpTy::apply_nils()?,
                (Value::Bool(lhs), Constant::Bool(rhs)) => OpTy::apply_bools(lhs, rhs)?,
                (Value::Number(lhs), Constant::Float(rhs)) => OpTy::apply_numbers(lhs, rhs.into()),
                (Value::Number(lhs), Constant::Integer(rhs)) => {
                    OpTy::apply_numbers(lhs, rhs.into())
                }
                (Value::String(lhs), Constant::String(rhs)) => {
                    OpTy::apply_strings(&*(*lhs).borrow(), &rhs)
                }
                // TODO: Metatables
                _ => false,
            }),
        );

        Ok(())
    }
}

impl<OpTy> ApplyBinop for BinOpData<CompareOpTag<OpTy>, Register, Register>
where
    OpTy: OpName + ComparisonOpEval,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        // TODO: metatables
        scopes.store(
            self.lhs,
            Value::Bool(match (scopes.load(self.lhs), scopes.load(self.rhs)) {
                (Value::Nil, Value::Nil) => OpTy::apply_nils()?,
                (Value::Bool(lhs), Value::Bool(rhs)) => OpTy::apply_bools(lhs, rhs)?,
                (Value::Number(lhs), Value::Number(rhs)) => OpTy::apply_numbers(lhs, rhs),
                (Value::String(lhs), Value::String(rhs)) => {
                    OpTy::apply_strings(&*(*lhs).borrow(), &*(*rhs).borrow())
                }
                // TODO: Metatables
                _ => false,
            }),
        );

        Ok(())
    }
}
