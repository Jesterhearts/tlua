use crate::{
    block::Block,
    combinators::parse_list0,
    expressions::Expression,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
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
    pub(crate) fn parse_remaining(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let cond = parse_cond_then(lexer, alloc)?;
        let body = Block::parse(lexer, alloc)?;

        let elif = parse_list0(lexer, alloc, ElseIf::try_parse)?;
        let else_final = lexer
            .next_if_eq(Token::KWelse)
            .map(|_| Block::parse_with_end(lexer, alloc))
            .map_or_else(
                || lexer.expecting_token(Token::KWend).map(|_| None),
                |block| block.map(Some),
            )?;

        Ok(Self {
            cond,
            body,
            elif,
            else_final,
        })
    }
}

impl<'chunk> ElseIf<'chunk> {
    fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        if lexer.next_if_eq(Token::KWelseif).is_none() {
            return Ok(None);
        }

        parse_cond_then(lexer, alloc)
            .and_then(|cond| Block::parse(lexer, alloc).map(|body| Some(Self { cond, body })))
    }
}

fn parse_cond_then<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Expression<'chunk>, ParseError> {
    Expression::parse(lexer, alloc)
        .and_then(|cond| lexer.expecting_token(Token::KWthen).map(|_| cond))
}
