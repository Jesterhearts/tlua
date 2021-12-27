use crate::ast::{
    block::Block,
    expressions::Expression,
};

#[derive(Debug, PartialEq)]
pub(crate) struct RepeatLoop<'chunk> {
    pub(crate) body: Block<'chunk>,
    pub(crate) terminator: Expression<'chunk>,
}
