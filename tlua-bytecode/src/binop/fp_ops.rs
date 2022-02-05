use derive_more::From;

use crate::{
    binop::{
        traits::{
            FloatBinop,
            NumericOpEval,
        },
        OpName,
    },
    AnonymousRegister,
    NumLike,
    Number,
    OpError,
};

/// Generic operation for anything that looks like a number, usable during
/// compilation

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
        #[derive(Clone, Copy, PartialEq, Eq, From)]
        pub struct $name {
            pub dst: AnonymousRegister,
            pub lhs: AnonymousRegister,
            pub rhs: AnonymousRegister,
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "{} {:?} {:?} {:?}",
                    Self::NAME,
                    self.dst,
                    self.lhs,
                    self.rhs
                )
            }
        }

        impl OpName for $name {
            const NAME: &'static str = paste::paste! { stringify!([< $name:snake >])};
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

        impl NumericOpEval for $name {
            fn evaluate<LHS, RHS>(lhs: LHS, rhs: RHS) -> Result<Number, OpError>
            where
                LHS: NumLike,
                RHS: NumLike,
            {
                if let (Some(lhs), Some(rhs)) = (lhs.as_int(), rhs.as_int()) {
                    Ok(Self::apply_ints(lhs, rhs))
                } else {
                    Ok(Self::apply_floats(
                        lhs.as_float()
                            .ok_or(OpError::InvalidType { op: Self::NAME })?,
                        rhs.as_float()
                            .ok_or(OpError::InvalidType { op: Self::NAME })?,
                    ))
                }
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
