use crate::{
    block::Block,
    expressions::Expression,
    lexer::Token,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
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
        lexer.expecting_token(Token::KWrepeat)?;

        let (body, _) = Block::parse(lexer, alloc).chain_or_recover_with(|| {
            lexer
                .expecting_token(Token::KWuntil)
                .mark_unrecoverable()
                .map(|_| ())
        })?;

        let terminator = Expression::parse(lexer, alloc).mark_unrecoverable()?;

        Ok(Self {
            body: body.unwrap_or_default(),
            terminator,
        })
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
