use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{
        map,
        opt,
    },
    sequence::{
        delimited,
        pair,
        preceded,
    },
};
use tracing::instrument;

use crate::{
    ast::expressions::{
        tables::{
            Field,
            TableConstructor,
        },
        Expression,
    },
    list::List,
    parsing::{
        identifiers::parse_identifier,
        lua_whitespace0,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

impl<'chunk> Parse<'chunk> for Field<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        alt((
            map(
                pair(
                    delimited(
                        pair(tag("["), lua_whitespace0),
                        |input| Expression::parse(input, alloc),
                        pair(lua_whitespace0, tag("]")),
                    ),
                    preceded(
                        delimited(lua_whitespace0, tag("="), lua_whitespace0),
                        |input| Expression::parse(input, alloc),
                    ),
                ),
                |(index, expression)| Self::Indexed { index, expression },
            ),
            map(
                pair(
                    |input| parse_identifier(input, alloc),
                    preceded(
                        delimited(lua_whitespace0, tag("="), lua_whitespace0),
                        |input| Expression::parse(input, alloc),
                    ),
                ),
                |(name, expression)| Self::Named { name, expression },
            ),
            map(
                preceded(lua_whitespace0, |input| Expression::parse(input, alloc)),
                |expression| Self::Arraylike { expression },
            ),
        ))(input)
    }
}

pub(crate) fn fields0<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, Field<'chunk>>> {
    let (remain, maybe_head) = opt(|input| Field::parse(input, alloc))(input)?;
    input = remain;

    let mut fields = List::default();

    let mut current = if let Some(head) = maybe_head {
        fields.cursor_mut().alloc_insert_advance(alloc, head)
    } else {
        let (remain, _) = opt(one_of(",;"))(input)?;
        return Ok((remain, fields));
    };

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, one_of(",;"), lua_whitespace0),
            |input| Field::parse(input, alloc),
        ))(input)?;
        input = remain;

        current = if let Some(next) = maybe_next {
            current.alloc_insert_advance(alloc, next)
        } else {
            break;
        };
    }

    let (remain, _) = opt(one_of(",;"))(input)?;

    Ok((remain, fields))
}

impl<'chunk> Parse<'chunk> for TableConstructor<'chunk> {
    #[instrument(level = "trace", name = "table", skip(input, alloc))]
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        map(
            delimited(
                pair(tag("{"), lua_whitespace0),
                |input| fields0(input, alloc),
                pair(lua_whitespace0, tag("}")),
            ),
            |fields| Self { fields },
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::TableConstructor;
    use crate::{
        ast::expressions::{
            tables::Field,
            Expression,
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            ASTAllocator,
            Parse,
            Span,
        },
        vm::Number,
    };

    #[test]
    pub(crate) fn parses_empty_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let (remain, result) = TableConstructor::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            TableConstructor {
                fields: Default::default()
            }
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_empty_table_semicolon() -> anyhow::Result<()> {
        let src = "{;}";

        let alloc = ASTAllocator::default();
        let (remain, result) = TableConstructor::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            TableConstructor {
                fields: Default::default()
            }
        );
        Ok(())
    }

    #[test]
    pub(crate) fn parses_arraylike_field() -> anyhow::Result<()> {
        let src = "10";

        let alloc = ASTAllocator::default();
        let (remain, result) = Field::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            Field::Arraylike {
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_named_field() -> anyhow::Result<()> {
        let src = "a = 10";

        let alloc = ASTAllocator::default();
        let (remain, result) = Field::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            Field::Named {
                name: "a".into(),
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_index_field() -> anyhow::Result<()> {
        let src = "[11] = 10";

        let alloc = ASTAllocator::default();
        let (remain, result) = Field::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            Field::Indexed {
                index: Expression::Number(Number::Integer(11)),
                expression: Expression::Number(Number::Integer(10))
            }
        );

        Ok(())
    }

    #[test]
    pub(crate) fn parses_parses_table_mixed() -> anyhow::Result<()> {
        let src = "{10, [11] = 12, a = 13; 14}";

        let alloc = ASTAllocator::default();
        let (remain, result) = TableConstructor::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            TableConstructor {
                fields: List::from_slice(&mut [
                    ListNode::new(Field::Arraylike {
                        expression: Expression::Number(Number::Integer(10))
                    }),
                    ListNode::new(Field::Indexed {
                        index: Expression::Number(Number::Integer(11)),
                        expression: Expression::Number(Number::Integer(12))
                    }),
                    ListNode::new(Field::Named {
                        name: "a".into(),
                        expression: Expression::Number(Number::Integer(13))
                    }),
                    ListNode::new(Field::Arraylike {
                        expression: Expression::Number(Number::Integer(14))
                    }),
                ])
            }
        );

        Ok(())
    }
}
