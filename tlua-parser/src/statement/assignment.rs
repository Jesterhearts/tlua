use crate::{
    combinators::parse_separated_list_with_head,
    expressions::Expression,
    lexer::Token,
    list::List,
    prefix_expression::{
        PrefixExpression,
        VarPrefixExpression,
    },
    ASTAllocator,
    ParseError,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub struct Assignment<'chunk> {
    pub varlist: List<'chunk, VarPrefixExpression<'chunk>>,
    pub expressions: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> Assignment<'chunk> {
    pub(crate) fn parse_remaining(
        head: VarPrefixExpression<'chunk>,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let varlist = parse_separated_list_with_head(
            head,
            lexer,
            alloc,
            |lexer, alloc| {
                let pos = lexer.current_span();
                PrefixExpression::try_parse(lexer, alloc).and_then(|expr| {
                    if let Some(PrefixExpression::Variable(v)) = expr {
                        Ok(Some(v))
                    } else {
                        Err(ParseError {
                            error: SyntaxError::ExpectedVariable,
                            location: crate::SourceSpan {
                                start: pos.start,
                                end: lexer.current_span().end,
                            },
                        })
                    }
                })
            },
            |token| *token == Token::Comma,
        )?;

        lexer.expecting_token(Token::Equals)?;

        let expressions = Expression::parse_list1(lexer, alloc)?;

        Ok(Self {
            varlist,
            expressions,
        })
    }
}
