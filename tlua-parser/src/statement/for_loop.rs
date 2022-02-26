use crate::{
    block::Block,
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
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
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let for_kw = lexer.next_if_eq(Token::KWfor).ok_or_else(|| {
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWfor))
        })?;

        let var = match Ident::parse(lexer, alloc) {
            Ok(ident) => ident,
            Err(e) => {
                lexer.reset(for_kw);
                return Err(e);
            }
        };

        lexer.next_if_eq(Token::Equals).ok_or_else(|| {
            lexer.reset(for_kw);
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::Equals))
        })?;

        let init = Expression::parse(lexer, alloc).mark_unrecoverable()?;
        lexer.next_if_eq(Token::Comma).ok_or_else(|| {
            ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::Equals))
        })?;
        let condition = Expression::parse(lexer, alloc).mark_unrecoverable()?;

        let increment = lexer
            .next_if_eq(Token::Comma)
            .ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::Comma))
            })
            .and_then(|_| Expression::parse(lexer, alloc))
            .recover()?;

        let body = Block::parse_do(lexer, alloc).mark_unrecoverable()?;

        Ok(Self {
            var,
            init,
            condition,
            increment,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::ForLoop;
    use crate::{
        expressions::{
            number::Number,
            Expression,
        },
        final_parser,
        identifiers::Ident,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_for() -> anyhow::Result<()> {
        let src = "for a = 0, 10 do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => ForLoop::parse)?;

        assert_eq!(
            result,
            ForLoop {
                var: Ident(0),
                init: Expression::Number(Number::Integer(0)),
                condition: Expression::Number(Number::Integer(10)),
                increment: None,
                body: Default::default()
            }
        );

        Ok(())
    }
}
