use derive_more::From;

use crate::{
    binop::{
        traits::{
            IntBinop,
            NumericOpEval,
        },
        OpName,
    },
    ImmediateRegister,
    NumLike,
    Number,
    OpError,
};

/// Converts an `f64` to an `i64` if it falls within the range of `i64` and has
/// no fractional component.
pub fn f64inbounds(f: f64) -> Result<i64, OpError> {
    if f > i64::MIN as f64 && f < i64::MAX as f64 && f.fract() == 0.0 {
        Ok(f as i64)
    } else {
        Err(OpError::FloatToIntConversionFailed { f })
    }
}

/// Generic operation for anything that looks like a number, usable during
/// compilation

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
        #[derive(Clone, Copy, PartialEq, Eq, From)]
        pub struct $name {
            pub dst: ImmediateRegister,
            pub lhs: ImmediateRegister,
            pub rhs: ImmediateRegister,
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

        impl IntBinop for $name {
            fn apply_ints(lhs: i64, rhs: i64) -> Number {
                let $lhs_int = lhs;
                let $rhs_int = rhs;

                $when_ints
            }
        }

        impl NumericOpEval for $name {
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
                            .ok_or(OpError::InvalidType { op: Self::NAME })?,
                    )?
                };

                let rhs = if let Some(rhs) = rhs.as_int() {
                    rhs
                } else {
                    f64inbounds(
                        rhs.as_float()
                            .ok_or(OpError::InvalidType { op: Self::NAME })?,
                    )?
                };

                Ok(Self::apply_ints(lhs, rhs))
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
