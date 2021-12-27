use crate::ast::{
    block::Block,
    expressions::Expression,
    identifiers::Ident,
};

#[derive(Debug, PartialEq)]
pub(crate) struct ForLoop<'chunk> {
    pub(crate) var: Ident,
    pub(crate) init: Expression<'chunk>,
    pub(crate) condition: Expression<'chunk>,
    pub(crate) increment: Option<Expression<'chunk>>,
    pub(crate) body: Block<'chunk>,
}
