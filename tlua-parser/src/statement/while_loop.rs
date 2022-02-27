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
pub struct WhileLoop<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> WhileLoop<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::KWwhile)?;

        let cond = Expression::parse(lexer, alloc).mark_unrecoverable()?;
        let body = Block::parse_do(lexer, alloc).mark_unrecoverable()?;

        Ok(Self { cond, body })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::WhileLoop;
    use crate::{
        expressions::Expression,
        final_parser,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_while() -> anyhow::Result<()> {
        let src = "while true do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => WhileLoop::parse)?;

        assert_eq!(
            result,
            WhileLoop {
                cond: Expression::Bool(true),
                body: Default::default()
            }
        );

        Ok(())
    }
}
