use crate::{
    block::Block,
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub struct ForEachLoop<'chunk> {
    pub vars: List<'chunk, Ident>,
    pub expressions: List<'chunk, Expression<'chunk>>,
    pub body: Block<'chunk>,
}

impl<'chunk> ForEachLoop<'chunk> {
    pub(crate) fn parse_remaining(
        head: Ident,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let vars = Ident::parse_list_with_head(head, lexer, alloc)?;

        lexer.expecting_token(Token::KWin)?;

        let expressions = Expression::parse_list1(lexer, alloc)?;
        let body = Block::parse_do(lexer, alloc)?;

        Ok(Self {
            vars,
            expressions,
            body,
        })
    }
}
