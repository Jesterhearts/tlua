use derive_more::{
    Deref,
    From,
    Into,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct ImmediateRegister(usize);

impl std::fmt::Debug for ImmediateRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("imm{}", self.0))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deref, From)]
pub struct MappedRegister<RegisterTy>(pub RegisterTy);

impl<RegisterTy> std::fmt::Debug for MappedRegister<RegisterTy>
where
    RegisterTy: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From)]
pub struct Register {
    pub scope: u16,
    pub offset: u16,
}

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("scope{}[{}]", self.scope, self.offset))
    }
}
