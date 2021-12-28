use std::marker::PhantomData;

mod bool_ops;
mod cmp_ops;
mod fp_ops;
mod int_ops;
pub mod traits;

use self::traits::*;
pub use self::{
    bool_ops::*,
    cmp_ops::*,
    fp_ops::*,
    int_ops::*,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinOp<OpTag, LhsTy, RhsTy> {
    lhs: LhsTy,
    rhs: RhsTy,

    _tag: PhantomData<OpTag>,
}

impl<OpTag, LhsTy, RhsTy> From<(LhsTy, RhsTy)> for BinOp<OpTag, LhsTy, RhsTy> {
    fn from((lhs, rhs): (LhsTy, RhsTy)) -> Self {
        Self {
            lhs,
            rhs,
            _tag: PhantomData::default(),
        }
    }
}

impl<OpTag, LhsTy, RhsTy> From<BinOp<OpTag, LhsTy, RhsTy>> for (LhsTy, RhsTy) {
    fn from(val: BinOp<OpTag, LhsTy, RhsTy>) -> Self {
        (val.lhs, val.rhs)
    }
}
