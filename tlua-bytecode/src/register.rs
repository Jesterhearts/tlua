use derive_more::{
    Deref,
    From,
    Into,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Into)]
pub struct AnonymousRegister(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct MappedRegister<RegisterTy>(pub RegisterTy);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct Register {
    pub scope: u16,
    pub offset: u16,
}
