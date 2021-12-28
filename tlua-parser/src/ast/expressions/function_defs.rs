use crate::{
    ast::{
        block::Block,
        identifiers::Ident,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub struct FnParams<'chunk> {
    /// Note that LUA 5.4 doesn't distinguish multiple variables during function
    /// evaluation. So a function like `(a, a) return a + a; end` when
    /// called with `(10, 11)` produces `22` in valid lua.
    pub named_params: List<'chunk, Ident>,
    pub varargs: bool,
}

#[derive(Debug, PartialEq)]
pub struct FnBody<'chunk> {
    pub params: FnParams<'chunk>,
    pub body: Block<'chunk>,
}

#[derive(Debug, PartialEq)]
pub struct FnName<'chunk> {
    pub path: List<'chunk, Ident>,
    pub method: Option<Ident>,
}
