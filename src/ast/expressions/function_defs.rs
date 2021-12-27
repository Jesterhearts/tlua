use crate::{
    ast::{
        block::Block,
        identifiers::Ident,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub(crate) struct FnParams<'chunk> {
    /// Note that LUA 5.4 doesn't distinguish multiple variables during function
    /// evaluation. So a function like `(a, a) return a + a; end` when
    /// called with `(10, 11)` produces `22` in valid lua.
    pub(crate) named_params: List<'chunk, Ident>,
    pub(crate) varargs: bool,
}

#[derive(Debug, PartialEq)]
pub(crate) struct FnBody<'chunk> {
    pub(crate) params: FnParams<'chunk>,
    pub(crate) body: Block<'chunk>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct FnName<'chunk> {
    pub(crate) path: List<'chunk, Ident>,
    pub(crate) method: Option<Ident>,
}
