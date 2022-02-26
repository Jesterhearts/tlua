use tlua_strings::LuaString;

use crate::{
    Number,
    StringLike,
};

impl StringLike for LuaString {
    fn as_lua_string_bytes(&self) -> &[u8] {
        (*self).as_slice()
    }
}

impl StringLike for &LuaString {
    fn as_lua_string_bytes(&self) -> &[u8] {
        (*self).as_lua_string_bytes()
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
