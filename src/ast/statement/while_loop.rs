use crate::ast::{
    block::Block,
    expressions::Expression,
};

#[derive(Debug, PartialEq)]
pub(crate) struct WhileLoop<'chunk> {
    pub(crate) cond: Expression<'chunk>,
    pub(crate) body: Block<'chunk>,
}
