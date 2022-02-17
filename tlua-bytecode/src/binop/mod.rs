mod bool_ops;
mod cmp_ops;
mod concat_ops;
mod fp_ops;
mod int_ops;
pub mod traits;

use self::traits::*;
pub use self::{
    bool_ops::*,
    cmp_ops::*,
    concat_ops::*,
    fp_ops::*,
    int_ops::*,
};

macro_rules! debug_binop {
    ($name:ident) => {
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {:?} {:?}", Self::NAME, self.lhs, self.rhs)
            }
        }
    };
}

use debug_binop;
