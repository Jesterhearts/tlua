use bstr::{
    BString,
    ByteSlice,
};
use derive_more::{
    Deref,
    DerefMut,
    From,
    Into,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deref, DerefMut, From, Into)]
pub struct LuaString(BString);

impl ::std::fmt::Debug for LuaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ::std::fmt::Display for LuaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ::std::borrow::Borrow<bstr::BStr> for LuaString {
    fn borrow(&self) -> &bstr::BStr {
        self.0.as_bstr()
    }
}

impl ::std::borrow::Borrow<[u8]> for LuaString {
    fn borrow(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl ::std::borrow::Borrow<str> for LuaString {
    fn borrow(&self) -> &str {
        std::str::from_utf8(self.0.as_bstr()).expect("Valid utf8")
    }
}

impl<'s> From<&'s LuaString> for &'s bstr::BStr {
    fn from(val: &'s LuaString) -> Self {
        val.0.as_bstr()
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
