use nom::{
    bytes::complete::tag,
    combinator::{
        map,
        opt,
    },
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
        tuple,
    },
};

use crate::{
    block::Block,
    expressions::Expression,
    list::List,
    lua_whitespace0,
    lua_whitespace1,
    ASTAllocator,
    Parse,
    ParseResult,
    Span,
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

impl<'chunk> Parse<'chunk> for If<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        delimited(
            pair(tag("if"), lua_whitespace1),
            map(
                tuple((
                    |input| parse_cond_then_body(input, alloc),
                    |input| elif0(input, alloc),
                    opt(preceded(
                        delimited(lua_whitespace0, tag("else"), lua_whitespace1),
                        |input| Block::parse(input, alloc),
                    )),
                )),
                |((cond, body), elif, else_final)| Self {
                    cond,
                    body,
                    elif,
                    else_final,
                },
            ),
            pair(lua_whitespace0, tag("end")),
        )(input)
    }
}

fn parse_cond_then_body<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, (Expression<'chunk>, Block<'chunk>)> {
    pair(
        terminated(
            |input| Expression::parse(input, alloc),
            delimited(lua_whitespace0, tag("then"), lua_whitespace0),
        ),
        |input| Block::parse(input, alloc),
    )(input)
}

fn elif0<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, ElseIf<'chunk>>> {
    let (remain, head) = opt(|input| parse_elif(input, alloc))(input)?;
    input = remain;

    let mut elifs = List::default();

    let mut current = if let Some(head) = head {
        elifs.cursor_mut().alloc_insert_advance(alloc, head)
    } else {
        return Ok((remain, List::default()));
    };

    loop {
        let (remain, next) = opt(|input| parse_elif(input, alloc))(input)?;
        input = remain;

        current = if let Some(next) = next {
            current.alloc_insert_advance(alloc, next)
        } else {
            return Ok((remain, elifs));
        };
    }
}

fn parse_elif<'src, 'chunk>(
    input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, ElseIf<'chunk>> {
    map(
        preceded(
            delimited(lua_whitespace0, tag("elseif"), lua_whitespace1),
            |input| parse_cond_then_body(input, alloc),
        ),
        |(cond, body)| ElseIf { cond, body },
    )(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::If;
    use crate::{
        expressions::Expression,
        list::{
            List,
            ListNode,
        },
        statement::if_statement::ElseIf,
        ASTAllocator,
        Parse,
        Span,
    };

    #[test]
    pub fn parses_if() -> anyhow::Result<()> {
        let src = "if true then end";

        let alloc = ASTAllocator::default();
        let (remain, result) = If::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = If::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
        let (remain, result) = If::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
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
