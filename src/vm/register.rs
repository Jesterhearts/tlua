use std::num::NonZeroU16;

use derive_more::From;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From)]
pub(crate) struct Register {
    pub(crate) scope: Option<NonZeroU16>,
    pub(crate) offset: u16,
}
