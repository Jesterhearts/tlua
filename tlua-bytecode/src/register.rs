use std::num::NonZeroU16;

use derive_more::From;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct Register {
    pub scope: Option<NonZeroU16>,
    pub offset: u16,
}
