use std::borrow::Borrow;

use tlua_bytecode::StringLike;
use tlua_parser::ast::{
    constant_string::ConstantString,
    identifiers::Ident,
};

#[derive(Clone, PartialEq, PartialOrd, Hash, Default)]
pub struct LuaString(Vec<u8>);

impl std::fmt::Debug for LuaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LuaString")
            .field(&String::from_utf8_lossy(&self.0))
            .finish()
    }
}

impl StringLike for LuaString {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<LuaString> for Ident {
    fn from(val: LuaString) -> Self {
        Ident::new_from_slice(val.0.as_slice())
    }
}

impl From<&str> for LuaString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<&[u8]> for LuaString {
    fn from(s: &[u8]) -> Self {
        Self::new_byte_slice(s)
    }
}

impl From<ConstantString> for LuaString {
    fn from(c: ConstantString) -> Self {
        Self::new_byte_slice(c.as_slice())
    }
}

impl LuaString {
    pub fn new(s: &str) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(s.as_bytes());
        Self::new_bytes(bytes)
    }

    pub fn new_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn new_byte_slice<T>(slice: T) -> Self
    where
        T: Borrow<[u8]>,
    {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(slice.borrow());
        Self::new_bytes(bytes)
    }
}

impl LuaString {
    pub fn extend(&mut self, s: &str) {
        self.extend_bytes(s.as_bytes())
    }

    pub fn extend_bytes(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }

    pub fn push(&mut self, c: u8) {
        self.0.push(c);
    }

    pub fn pop(&mut self) -> Option<u8> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl PartialEq<&str> for LuaString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

pub struct Thread {/* TODO(lang-5.4): This is basically rust's async state machine */}

pub enum MetaMethod {
    Add(()),
    Sub(()),
    Mul(()),
    Div(()),
    Mod(()),
    Pow(()),
    Unm(()),
    Idiv(()),
    Band(()),
    Bor(()),
    Bxor(()),
    Bnot(()),
    Shl(()),
    Shr(()),
    Concat(()),
    Len(()),
    Eq(()),
    Lt(()),
    Le(()),
    Index(()),
    NewIndex(()),
    Call(()),
    Gc(()),
    Close(()),
}
