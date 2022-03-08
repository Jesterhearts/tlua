use crate::{
    combinators::parse_separated_list_with_head,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    PeekableLexer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ident(pub(crate) usize);

impl Ident {
    pub(crate) fn parse(lexer: &mut PeekableLexer, _: &ASTAllocator) -> Result<Self, ParseError> {
        lexer
            .expecting_token(Token::Ident)
            .map(|ident| lexer.strings.add_ident(ident.src))
    }

    pub(crate) fn try_parse(lexer: &mut PeekableLexer, _: &ASTAllocator) -> Option<Self> {
        lexer
            .next_if_eq(Token::Ident)
            .map(|ident| lexer.strings.add_ident(ident.src))
    }

    pub(crate) fn parse_list_with_head<'chunk>(
        head: Ident,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        parse_separated_list_with_head(
            head,
            lexer,
            alloc,
            |lexer, alloc| Ok(Self::try_parse(lexer, alloc)),
            |token| *token == Token::Comma,
        )
    }
}
