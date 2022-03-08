use crate::{
    block::Block,
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    ASTAllocator,
    ParseError,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub struct ForLoop<'chunk> {
    pub var: Ident,
    pub init: Expression<'chunk>,
    pub condition: Expression<'chunk>,
    pub increment: Option<Expression<'chunk>>,
    pub body: Block<'chunk>,
}

impl<'chunk> ForLoop<'chunk> {
    pub(crate) fn parse_remaining(
        var: Ident,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::Equals)?;

        let init = Expression::parse(lexer, alloc)?;
        lexer.expecting_token(Token::Comma)?;

        let condition = Expression::parse(lexer, alloc)?;

        let increment = lexer
            .next_if_eq(Token::Comma)
            .map_or(Ok(None), |_| Expression::try_parse(lexer, alloc))?;

        let body = Block::parse_do(lexer, alloc)?;

        Ok(Self {
            var,
            init,
            condition,
            increment,
            body,
        })
    }
}
