use crate::{
    block::Block,
    expressions::Expression,
    ASTAllocator,
    ParseError,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub struct WhileLoop<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> WhileLoop<'chunk> {
    pub(crate) fn parse_remaining(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        Expression::parse(lexer, alloc)
            .and_then(|cond| Block::parse_do(lexer, alloc).map(|body| Self { cond, body }))
    }
}
