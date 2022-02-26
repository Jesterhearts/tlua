use crate::{
    block::Block,
    expressions::Expression,
    lexer::Token,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub struct RepeatLoop<'chunk> {
    pub body: Block<'chunk>,
    pub terminator: Expression<'chunk>,
}

impl<'chunk> RepeatLoop<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.next_if_eq(Token::KWrepeat).ok_or_else(|| {
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWrepeat))
        })?;

        let body = Block::parse(lexer, alloc).mark_unrecoverable()?;
        lexer.next_if_eq(Token::KWuntil).ok_or_else(|| {
            ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWuntil))
        })?;

        let terminator = Expression::parse(lexer, alloc).mark_unrecoverable()?;

        Ok(Self { body, terminator })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::RepeatLoop;
    use crate::{
        expressions::Expression,
        final_parser,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_repeat() -> anyhow::Result<()> {
        let src = "repeat until true";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => RepeatLoop::parse)?;

        assert_eq!(
            result,
            RepeatLoop {
                body: Default::default(),
                terminator: Expression::Bool(true)
            }
        );

        Ok(())
    }
}
