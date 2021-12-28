use crate::{
    ast::{
        block::Block,
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub struct ForEachLoop<'chunk> {
    pub vars: List<'chunk, Ident>,
    pub expressions: List<'chunk, Expression<'chunk>>,
    pub body: Block<'chunk>,
}
