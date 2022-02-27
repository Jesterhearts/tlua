use crate::{
    lexer::Token,
    list::List,
    parse_separated_list1,
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

    pub(crate) fn parse_list1<'chunk>(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        parse_separated_list1(lexer, alloc, Self::parse, |token| *token == Token::Comma)
    }
}
