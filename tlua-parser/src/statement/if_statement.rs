use crate::{
    block::Block,
    expressions::Expression,
    lexer::Token,
    list::List,
    parse_list1,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
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
        lexer.expecting_token(Token::KWif)?;

        let cond = parse_cond_then(lexer, alloc)?;

        let (body, (elif, else_final)) = Block::parse(lexer, alloc)
            .chain_or_recover_with(|| {
                ElseIf::parse_list1(lexer, alloc)
                    .chain_or_recover_with(|| {
                        parse_else_final(lexer, alloc)
                            .map(Some)
                            .recover_with(|| lexer.expecting_token(Token::KWend).map(|_| None))
                    })
                    .map(|(elifs, else_final)| (elifs.unwrap_or_default(), else_final))
            })
            .map(|(body, rest)| (body.unwrap_or_default(), rest))
            .mark_unrecoverable()?;

        Ok(Self {
            cond,
            body,
            elif,
            else_final,
        })
    }
}

impl<'chunk> ElseIf<'chunk> {
    fn parse_list1(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<List<'chunk, Self>, ParseError> {
        parse_list1(lexer, alloc, |lexer, alloc| {
            lexer.expecting_token(Token::KWelseif)?;
            parse_cond_then(lexer, alloc).and_then(|cond| {
                Block::parse(lexer, alloc).recover().map(|body| ElseIf {
                    cond,
                    body: body.unwrap_or_default(),
                })
            })
        })
    }
}

fn parse_else_final<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Block<'chunk>, ParseError> {
    lexer
        .expecting_token(Token::KWelse)
        .and_then(|_| Block::parse_with_end(lexer, alloc))
}

fn parse_cond_then<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Expression<'chunk>, ParseError> {
    let cond = Expression::parse(lexer, alloc).mark_unrecoverable()?;
    lexer.expecting_token(Token::KWthen).mark_unrecoverable()?;
    Ok(cond)
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
