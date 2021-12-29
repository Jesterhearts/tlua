use std::marker::PhantomData;

use derive_more::{
    Deref,
    From,
};

use crate::{
    binop::{
        traits::{
            FloatBinop,
            NumericOpEval,
        },
        BinOpData,
        OpName,
    },
    NumLike,
    Number,
    OpError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatOpTag<OpTy: FloatBinop>(PhantomData<OpTy>);

/// Generic operation for anything that looks like a number, usable during
/// compilation
impl<OpTy, LhsTy, RhsTy> NumericOpEval for BinOpData<FloatOpTag<OpTy>, LhsTy, RhsTy>
where
    OpTy: FloatBinop + OpName,
{
    fn evaluate<LHS, RHS>(lhs: LHS, rhs: RHS) -> Result<Number, OpError>
    where
        LHS: NumLike,
        RHS: NumLike,
    {
        if let (Some(lhs), Some(rhs)) = (lhs.as_int(), rhs.as_int()) {
            Ok(OpTy::apply_ints(lhs, rhs))
        } else {
            Ok(OpTy::apply_floats(
                lhs.as_float()
                    .ok_or(OpError::InvalidType { op: OpTy::NAME })?,
                rhs.as_float()
                    .ok_or(OpError::InvalidType { op: OpTy::NAME })?,
            ))
        }
    }
}

// TODO(cleanup): This could probably share some macro code with the other
// binop_impls
macro_rules! float_binop_impl {
    (
        $name:ident =>
        {
            ($lhs_int:ident : int, $rhs_int:ident : int) => $when_ints:expr,
            ($lhs_float:ident : float, $rhs_float:ident : float) => $when_floats:expr $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, From, Deref)]
        pub struct $name<LhsTy, RhsTy>(BinOpData<FloatOpTag<Self>, LhsTy, RhsTy>);

        impl<LhsTy, RhsTy> From<(LhsTy, RhsTy)> for $name<LhsTy, RhsTy> {
            fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
                Self(BinOpData {
                    lhs,
                    rhs,
                    _tag: Default::default(),
                })
            }
        }

        impl<LhsTy, RhsTy> OpName for $name<LhsTy, RhsTy> {
            const NAME: &'static str = stringify!($name);
        }

        impl<LhsTy, RhsTy> NumericOpEval for $name<LhsTy, RhsTy> {
            fn evaluate<LHS, RHS>(lhs: LHS, rhs: RHS) -> Result<Number, OpError>
            where
                LHS: NumLike,
                RHS: NumLike,
            {
                BinOpData::<FloatOpTag<Self>, LhsTy, RhsTy>::evaluate(lhs, rhs)
            }
        }

        impl<LhsTy, RhsTy> FloatBinop for $name<LhsTy, RhsTy> {
            fn apply_ints(lhs: i64, rhs: i64) -> Number {
                let $lhs_int = lhs;
                let $rhs_int = rhs;

                $when_ints
            }

            fn apply_floats(lhs: f64, rhs: f64) -> Number {
                let $lhs_float = lhs;
                let $rhs_float = rhs;

                $when_floats
            }
        }
    };
}

macro_rules! float_binop {
    (
        $name:ident =>
        {
            ($lhs_int:ident : int, $rhs_int:ident : int) => $when_ints:expr,
            ($lhs_float:ident : float, $rhs_float:ident : float) => $when_floats:expr $(,)?
        }
    ) => {
        float_binop_impl! {
            $name => {
                ($lhs_int : int, $rhs_int : int) => $when_ints,
                ($lhs_float : float, $rhs_float : float) => $when_floats
            }
        }

        paste::paste! {
            float_binop_impl! {
                [< $name Indirect >] => {
                    ($lhs_int : int, $rhs_int : int) => $when_ints,
                    ($lhs_float : float, $rhs_float : float) => $when_floats
                }
            }
        }
    };
}

float_binop!(Add => {
    (lhs: int, rhs: int) =>  Number::Integer(lhs.wrapping_add(rhs)),
    (lhs: float, rhs: float) => Number::Float(lhs + rhs),
});

float_binop!(Subtract => {
    (lhs: int, rhs: int) =>  Number::Integer(lhs.wrapping_sub(rhs)),
    (lhs: float, rhs: float) => Number::Float(lhs - rhs),
});

float_binop!(Times => {
    (lhs: int, rhs: int) =>  Number::Integer(lhs.wrapping_mul(rhs)),
    (lhs: float, rhs: float) => Number::Float(lhs * rhs),
});

float_binop!(Divide => {
    (lhs: int, rhs: int) => Number::Float(lhs as f64 / rhs as f64) ,
    (lhs: float, rhs: float) => Number::Float(lhs / rhs),
});

float_binop!(IDiv => {
    (lhs: int, rhs: int) =>  Number::Integer(lhs / rhs),
    (lhs: float, rhs: float) => Number::Float((lhs + rhs).floor()),
});

float_binop!(Modulo => {
    (lhs: int, rhs: int) =>  Number::Integer(lhs % rhs),
    (lhs: float, rhs: float) => Number::Float(lhs % rhs),
});

float_binop!(Exponetiation => {
    (lhs: int, rhs: int) =>  Number::Float((lhs as f64).powf(rhs as f64)),
    (lhs: float, rhs: float) => Number::Float(lhs.powf(rhs)),
});
