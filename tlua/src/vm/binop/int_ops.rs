use tlua_bytecode::{
    binop::{
        traits::{
            IntBinop,
            NumericOpEval,
            OpName,
        },
        IntOp,
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

impl<OpTy> ApplyBinop for IntOp<OpTy, Register>
where
    OpTy: OpName + IntBinop,
{
    fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
        scopes.store(
            self.lhs,
            match scopes.load(self.lhs) {
                Value::Number(lhs) => Value::Number(Self::evaluate(
                    lhs,
                    Value::try_from(self.rhs).unwrap_or_else(|reg| scopes.load(reg)),
                )?),
                Value::Table(_) => {
                    todo!("metatables are not supported");
                }
                _ => return Err(OpError::InvalidType { op: OpTy::NAME }),
            },
        );

        Ok(())
    }
}
