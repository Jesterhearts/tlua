use tlua_bytecode::{
    binop::{
        traits::{
            IntBinop,
            NumericOpEval,
            OpName,
        },
        BinOpData,
        IntOpTag,
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

impl<OpTy> ApplyBinop for BinOpData<IntOpTag<OpTy>, Register, Constant>
where
    OpTy: OpName + IntBinop,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        scopes.store(
            self.lhs,
            match scopes.load(self.lhs) {
                Value::Number(lhs) => Value::Number(Self::evaluate(lhs, self.rhs)?),
                Value::Table(_) => {
                    todo!("metatables are not supported");
                }
                _ => return Err(OpError::InvalidType { op: OpTy::NAME }),
            },
        );

        Ok(())
    }
}

impl<OpTy> ApplyBinop for BinOpData<IntOpTag<OpTy>, Register, Register>
where
    OpTy: OpName + IntBinop,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        scopes.store(
            self.lhs,
            match (scopes.load(self.lhs), scopes.load(self.rhs)) {
                (Value::Table(_), _rhs) => todo!("metatables are not supported"),
                (_lhs, Value::Table(_)) => todo!("metatables are not supported"),
                (Value::Number(lhs), Value::Number(rhs)) => {
                    Value::Number(Self::evaluate(lhs, rhs)?)
                }
                _ => return Err(OpError::InvalidType { op: OpTy::NAME }),
            },
        );

        Ok(())
    }
}
