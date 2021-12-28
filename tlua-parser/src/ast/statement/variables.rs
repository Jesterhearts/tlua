use crate::{
    ast::{
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Attribute {
    Const,
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVar {
    pub name: Ident,
    pub attribute: Option<Attribute>,
}

#[derive(Debug, PartialEq)]
pub struct LocalVarList<'chunk> {
    pub vars: List<'chunk, LocalVar>,
    pub initializers: List<'chunk, Expression<'chunk>>,
}
