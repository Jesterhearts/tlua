use crate::{
    block::Block,
    expressions::Expression,
    lexer::Token,
    list::List,
    parse_list0,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, PartialEq)]
pub struct If<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
    pub elif: List<'chunk, ElseIf<'chunk>>,
    pub else_final: Option<Block<'chunk>>,
}

#[derive(Debug, PartialEq)]
pub struct ElseIf<'chunk> {
    pub cond: Expression<'chunk>,
    pub body: Block<'chunk>,
}

impl<'chunk> If<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.next_if_eq(Token::KWif).ok_or_else(|| {
            ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWif))
        })?;

        let (cond, body) = parse_cond_then_body(lexer, alloc).mark_unrecoverable()?;

        let elif = parse_list0(lexer, alloc, |lexer, alloc| {
            lexer.next_if_eq(Token::KWelseif).ok_or_else(|| {
                ParseError::recoverable_from_here(
                    lexer,
                    SyntaxError::ExpectedToken(Token::KWelseif),
                )
            })?;
            parse_cond_then_body(lexer, alloc).map(|(cond, body)| ElseIf { cond, body })
        })?;

        let else_final = lexer
            .next_if_eq(Token::KWelse)
            .ok_or_else(|| {
                ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWelse))
            })
            .and_then(|_| Block::parse(lexer, alloc))
            .recover()?;

        lexer.next_if_eq(Token::KWend).ok_or_else(|| {
            ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWend))
        })?;

        Ok(Self {
            cond,
            body,
            elif,
            else_final,
        })
    }
}

fn parse_cond_then_body<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<(Expression<'chunk>, Block<'chunk>), ParseError> {
    let cond = Expression::parse(lexer, alloc).mark_unrecoverable()?;
    lexer.next_if_eq(Token::KWthen).ok_or_else(|| {
        ParseError::unrecoverable_from_here(lexer, SyntaxError::ExpectedToken(Token::KWthen))
    })?;
    let body = Block::parse(lexer, alloc).mark_unrecoverable()?;

    Ok((cond, body))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::If;
    use crate::{
        expressions::Expression,
        final_parser,
        list::{
            List,
            ListNode,
        },
        statement::if_statement::ElseIf,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_if() -> anyhow::Result<()> {
        let src = "if true then end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => If::parse)?;

        assert_eq!(
            result,
            If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: Default::default(),
                else_final: None,
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_if_else() -> anyhow::Result<()> {
        let src = "if true then else end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => If::parse)?;

        assert_eq!(
            result,
            If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: Default::default(),
                else_final: Some(Default::default())
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_if_elseif_else() -> anyhow::Result<()> {
        let src = "if true then elseif true then else end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => If::parse)?;

        assert_eq!(
            result,
            If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: List::new(&mut ListNode::new(ElseIf {
                    cond: Expression::Bool(true),
                    body: Default::default(),
                })),
                else_final: Some(Default::default())
            }
        );

        Ok(())
    }
}
