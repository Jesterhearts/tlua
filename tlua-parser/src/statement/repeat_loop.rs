use crate::{
    block::Block,
    expressions::Expression,
    lexer::Token,
    ASTAllocator,
    ParseError,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub struct RepeatLoop<'chunk> {
    pub body: Block<'chunk>,
    pub terminator: Expression<'chunk>,
}

impl<'chunk> RepeatLoop<'chunk> {
    pub(crate) fn parse_remaining(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        Block::parse(lexer, alloc).and_then(|body| {
            lexer.expecting_token(Token::KWuntil)?;
            Expression::parse(lexer, alloc).map(|terminator| Self { body, terminator })
        })
    }
}
