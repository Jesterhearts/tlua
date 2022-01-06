use crate::{
    binop::{
        traits::{
            FloatBinop,
            NumericOpEval,
        },
        OpName,
    },
    encoding::{
        EncodableInstruction,
        InstructionTag,
    },
    NumLike,
    Number,
    OpError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatOp<OpTy: FloatBinop, LhsTy, RhsTy> {
    pub lhs: LhsTy,
    pub rhs: RhsTy,
    op: OpTy,
}

impl<OpTy, LhsTy, RhsTy> From<FloatOp<OpTy, LhsTy, RhsTy>> for (LhsTy, RhsTy)
where
    OpTy: FloatBinop,
{
    fn from(val: FloatOp<OpTy, LhsTy, RhsTy>) -> Self {
        (val.lhs, val.rhs)
    }
}

impl<OpTy, LhsTy, RhsTy> From<(LhsTy, RhsTy)> for FloatOp<OpTy, LhsTy, RhsTy>
where
    OpTy: FloatBinop + Default,
{
    fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
        Self {
            lhs,
            rhs,
            op: Default::default(),
        }
    }
}

/// Generic operation for anything that looks like a number, usable during
/// compilation
impl<OpTy, LhsTy, RhsTy> NumericOpEval for FloatOp<OpTy, LhsTy, RhsTy>
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
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct $name;

        impl EncodableInstruction for $name {
            const TAG: InstructionTag = InstructionTag::$name;
        }

        impl OpName for $name {
            const NAME: &'static str = stringify!($name);
        }

        impl FloatBinop for $name {
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
    (lhs: float, rhs: float) => Number::Float((lhs / rhs).floor()),
});

float_binop!(Modulo => {
    (lhs: int, rhs: int) =>  Number::Integer(lhs % rhs),
    (lhs: float, rhs: float) => Number::Float(lhs % rhs),
});

float_binop!(Exponetiation => {
    (lhs: int, rhs: int) =>  Number::Float((lhs as f64).powf(rhs as f64)),
    (lhs: float, rhs: float) => Number::Float(lhs.powf(rhs)),
});
