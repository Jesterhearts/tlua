use crate::{
    ast::{
        block::Block,
        expressions::Expression,
        identifiers::Ident,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub(crate) struct ForEachLoop<'chunk> {
    pub(crate) vars: List<'chunk, Ident>,
    pub(crate) expressions: List<'chunk, Expression<'chunk>>,
    pub(crate) body: Block<'chunk>,
}
