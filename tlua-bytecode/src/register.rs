use std::num::NonZeroU16;

use bytemuck::{
    Pod,
    Zeroable,
};
use derive_more::From;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Pod, Zeroable)]
#[repr(C)]
pub struct Register {
    pub scope: Option<NonZeroU16>,
    pub offset: u16,
}
