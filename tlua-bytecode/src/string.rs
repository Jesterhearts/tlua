use bstr::BString;
use derive_more::{
    Deref,
    DerefMut,
};
use tlua_parser::{
    identifiers::Ident,
    string::ConstantString,
};

use crate::{
    Number,
    StringLike,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deref, DerefMut)]
pub struct LuaString(BString);

impl std::fmt::Debug for LuaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StringLike for LuaString {
    fn as_lua_string_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl StringLike for &LuaString {
    fn as_lua_string_bytes(&self) -> &[u8] {
        (*self).as_lua_string_bytes()
    }
}

impl From<LuaString> for Ident {
    fn from(val: LuaString) -> Self {
        Ident::new_from_slice(val.0.as_slice())
    }
}

impl From<&[u8]> for LuaString {
    fn from(s: &[u8]) -> Self {
        Self(BString::from(s))
    }
}

impl From<f64> for LuaString {
    fn from(f: f64) -> Self {
        Self(f.to_string().into())
    }
}

impl From<i64> for LuaString {
    fn from(i: i64) -> Self {
        Self(i.to_string().into())
    }
}

impl From<&str> for LuaString {
    fn from(s: &str) -> Self {
        Self(BString::from(s))
    }
}

impl From<ConstantString> for LuaString {
    fn from(c: ConstantString) -> Self {
        Self::from(c.as_slice())
    }
}

impl From<LuaString> for ConstantString {
    fn from(l: LuaString) -> Self {
        ConstantString::new(l.to_vec())
    }
}

impl From<&Number> for LuaString {
    fn from(num: &Number) -> Self {
        match *num {
            Number::Float(f) => f.into(),
            Number::Integer(i) => i.into(),
        }
    }
}
