use crate::{
    ast::{
        block::Block,
        expressions::Expression,
    },
    list::List,
};

#[derive(Debug, PartialEq)]
pub(crate) struct If<'chunk> {
    pub(crate) cond: Expression<'chunk>,
    pub(crate) body: Block<'chunk>,
    pub(crate) elif: List<'chunk, ElseIf<'chunk>>,
    pub(crate) else_final: Option<Block<'chunk>>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ElseIf<'chunk> {
    pub(crate) cond: Expression<'chunk>,
    pub(crate) body: Block<'chunk>,
}
