use crate::ast::{
    block::Block,
    expressions::Expression,
};

#[derive(Debug, PartialEq)]
pub struct WhileLoop<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}
