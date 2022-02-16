use derive_more::From;

use crate::{
    binop::{
        traits::ConcatBinop,
        OpName,
    },
    ImmediateRegister,
    LuaString,
    StringLike,
};

#[derive(Clone, Copy, PartialEq, Eq, From)]
pub struct Concat {
    pub dst: ImmediateRegister,
    pub lhs: ImmediateRegister,
    pub rhs: ImmediateRegister,
}

impl ::std::fmt::Debug for Concat {
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

impl OpName for Concat {
    const NAME: &'static str = "concat";
}

impl ConcatBinop for Concat {
    fn evaluate<Res: From<LuaString>, Lhs: StringLike, Rhs: StringLike>(lhs: Lhs, rhs: Rhs) -> Res {
        let mut res = LuaString::default();
        res.reserve_exact(lhs.as_lua_string_bytes().len() + rhs.as_lua_string_bytes().len());
        res.extend(lhs.as_lua_string_bytes());
        res.extend(rhs.as_lua_string_bytes());
        res.into()
    }
}
