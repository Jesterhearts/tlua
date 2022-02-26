use crate::{
    expressions::Expression,
    lexer::Token,
    list::List,
    parse_list_with_head,
    prefix_expression::{
        PrefixExpression,
        VarPrefixExpression,
    },
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub struct Assignment<'chunk> {
    pub varlist: List<'chunk, VarPrefixExpression<'chunk>>,
    pub expressions: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> Assignment<'chunk> {
    pub(crate) fn parse(
        head: VarPrefixExpression<'chunk>,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let varlist = parse_list_with_head(head, lexer, alloc, |lexer, alloc| {
            lexer.next_if_eq(Token::Comma).ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::Comma))
            })?;
            match PrefixExpression::parse(lexer, alloc).mark_unrecoverable()? {
                PrefixExpression::Variable(var) => Ok(var),
                _ => Err(ParseError::unrecoverable_from_here(
                    lexer,
                    SyntaxError::ExpectedVariable,
                )),
            }
        })
        .mark_unrecoverable()?;

        lexer.next_if_eq(Token::Equals).ok_or_else(|| {
            ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::Equals))
        })?;

        let expressions = Expression::parse_list1(lexer, alloc).mark_unrecoverable()?;

        Ok(Self {
            varlist,
            expressions,
        })
    }
}
