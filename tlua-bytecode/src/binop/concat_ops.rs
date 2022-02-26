use derive_more::From;
use tlua_strings::LuaString;

use crate::{
    binop::{
        debug_binop,
        traits::ConcatBinop,
        OpName,
    },
    ImmediateRegister,
    StringLike,
};

#[derive(Clone, Copy, PartialEq, Eq, From)]
pub struct Concat {
    pub lhs: ImmediateRegister,
    pub rhs: ImmediateRegister,
}

debug_binop! {Concat}

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
