use crate::{
    ast::{
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Attribute {
    Const,
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LocalVar {
    pub(crate) name: Ident,
    pub(crate) attribute: Option<Attribute>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct LocalVarList<'chunk> {
    pub(crate) vars: List<'chunk, LocalVar>,
    pub(crate) initializers: List<'chunk, Expression<'chunk>>,
}
