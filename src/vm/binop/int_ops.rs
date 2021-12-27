use std::marker::PhantomData;

use derive_more::{
    Deref,
    From,
};

use crate::{
    values::MetaMethod,
    vm::{
        binop::{
            traits::{
                ApplyBinop,
                IntBinop,
                NumericOpEval,
            },
            BinOp,
            OpName,
        },
        runtime::value::{
            function::ScopeSet,
            NumLike,
        },
        Constant,
        Number,
        OpError,
        Register,
        Value,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct IntOpTag<OpTy: IntBinop>(PhantomData<OpTy>);

pub(crate) fn f64inbounds(f: f64) -> Result<i64, OpError> {
    if f > i64::MIN as f64 && f < i64::MAX as f64 && f.fract() == 0.0 {
        Ok(f as i64)
    } else {
        Err(OpError::FloatToIntConversionFailed { f })
    }
}

/// Generic operation for anything that looks like a number, usable during
/// compilation
impl<OpTy, LhsTy, RhsTy> NumericOpEval for BinOp<IntOpTag<OpTy>, LhsTy, RhsTy>
where
    OpTy: IntBinop + OpName,
{
    fn evaluate<LHS, RHS>(lhs: LHS, rhs: RHS) -> Result<Number, OpError>
    where
        LHS: NumLike,
        RHS: NumLike,
    {
        let lhs = if let Some(lhs) = lhs.as_int() {
            lhs
        } else {
            f64inbounds(
                lhs.as_float()
                    .ok_or(OpError::InvalidType { op: OpTy::NAME })?,
            )?
        };

        let rhs = if let Some(rhs) = rhs.as_int() {
            rhs
        } else {
            f64inbounds(
                rhs.as_float()
                    .ok_or(OpError::InvalidType { op: OpTy::NAME })?,
            )?
        };

        Ok(OpTy::apply_ints(lhs, rhs))
    }
}

impl<OpTy> ApplyBinop for BinOp<IntOpTag<OpTy>, Register, Constant>
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

impl<OpTy> ApplyBinop for BinOp<IntOpTag<OpTy>, Register, Register>
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

fn shift_left(lhs: i64, rhs: i64) -> i64 {
    if rhs < -64 || rhs > 64 {
        0
    } else if rhs == 0 {
        lhs
    } else if rhs < 0 {
        lhs.wrapping_shr(rhs as u32)
    } else {
        lhs.wrapping_shl(rhs as u32)
    }
}

fn shift_right(lhs: i64, rhs: i64) -> i64 {
    if rhs < -64 || rhs > 64 {
        0
    } else if rhs == 0 {
        lhs
    } else if rhs < 0 {
        lhs.wrapping_shl(rhs as u32)
    } else {
        lhs.wrapping_shr(rhs as u32)
    }
}

macro_rules! int_binop_impl {
    (
        $name:ident =>
        {
            ($lhs_int:ident : int, $rhs_int:ident : int) => $when_ints:expr,
            ($lhs_float:ident : float, $rhs_float:ident : float) => $when_floats:expr $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
        pub(crate) struct $name<LhsTy, RhsTy>(BinOp<IntOpTag<Self>, LhsTy, RhsTy>);

        impl<LhsTy, RhsTy> From<(LhsTy, RhsTy)> for $name<LhsTy, RhsTy> {
            fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
                Self(BinOp {
                    lhs,
                    rhs,
                    _tag: Default::default(),
                })
            }
        }

        impl<LhsTy, RhsTy> OpName for $name<LhsTy, RhsTy> {
            const NAME: &'static str = stringify!($name);
        }

        impl ApplyBinop for $name<Register, Constant> {
            fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
                self.0.apply(scopes)
            }
        }

        impl ApplyBinop for $name<Register, Register> {
            fn apply(&self, scopes: &mut ScopeSet) -> Result<(), OpError> {
                self.0.apply(scopes)
            }
        }

        impl<LhsTy, RhsTy> NumericOpEval for $name<LhsTy, RhsTy> {
            fn evaluate<LHS, RHS>(lhs: LHS, rhs: RHS) -> Result<Number, OpError>
            where
                LHS: NumLike,
                RHS: NumLike,
            {
                BinOp::<IntOpTag<Self>, LhsTy, RhsTy>::evaluate(lhs, rhs)
            }
        }

        impl<LhsTy, RhsTy> IntBinop for $name<LhsTy, RhsTy> {
            fn apply_ints(lhs: i64, rhs: i64) -> Number {
                let $lhs_int = lhs;
                let $rhs_int = rhs;

                $when_ints
            }

            fn metamethod() -> MetaMethod {
                todo!("metamethods are not supported yet")
            }
        }
    };
}

macro_rules! int_binop {
    (
        $name:ident =>
        {
            ($lhs_int:ident : int, $rhs_int:ident : int) => $when_ints:expr,
            ($lhs_float:ident : float, $rhs_float:ident : float) => $when_floats:expr $(,)?
        }
    ) => {
        int_binop_impl! {
            $name => {
                ($lhs_int : int, $rhs_int : int) => $when_ints,
                ($lhs_float : float, $rhs_float : float) => $when_floats
            }
        }

        paste::paste! {
            int_binop_impl! {
                [< $name Indirect >] => {
                    ($lhs_int : int, $rhs_int : int) => $when_ints,
                    ($lhs_float : float, $rhs_float : float) => $when_floats
                }
            }
        }
    };
}

int_binop!(BitAnd => {
    (lhs: int, rhs: int) => Number::Integer(lhs & rhs),
    (lhs: float, rhs: float) => Number::Integer(lhs & rhs),
});

int_binop!(BitOr => {
    (lhs: int, rhs: int) => Number::Integer(lhs | rhs),
    (lhs: float, rhs: float) => Number::Integer(lhs | rhs),
});

int_binop!(BitXor => {
    (lhs: int, rhs: int) => Number::Integer(lhs ^ rhs),
    (lhs: float, rhs: float) => Number::Integer(lhs ^ rhs),
});

int_binop!(ShiftLeft => {
    (lhs: int, rhs: int) => Number::Integer(shift_left(lhs, rhs)),
    (lhs: float, rhs: float) => Number::Integer(shift_left(lhs, rhs)),
});

int_binop!(ShiftRight => {
    (lhs: int, rhs: int) => Number::Integer(shift_right(lhs, rhs)),
    (lhs: float, rhs: float) => Number::Integer(shift_right(lhs, rhs)),
});
