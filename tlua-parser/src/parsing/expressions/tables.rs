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
    ast::{
        expressions::{
            tables::TableConstructor,
            Expression,
        },
        identifiers::Ident,
    },
    parsing::{
        identifiers::parse_identifier,
        lua_whitespace0,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
    },
};

#[derive(Debug, PartialEq)]
enum Field<'chunk> {
    Arraylike {
        expression: Expression<'chunk>,
    },
    Named {
        name: Ident,
        expression: Expression<'chunk>,
    },
    Indexed {
        index: Expression<'chunk>,
        expression: Expression<'chunk>,
    },
}

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

impl<'chunk> Parse<'chunk> for TableConstructor<'chunk> {
    #[instrument(level = "trace", name = "table", skip(input, alloc))]
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        delimited(
            pair(tag("{"), lua_whitespace0),
            |input| parse_table_ctor(input, alloc),
            pair(lua_whitespace0, tag("}")),
        )(input)
    }
}

fn parse_table_ctor<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, TableConstructor<'chunk>> {
    let (remain, maybe_head) = opt(|input| Field::parse(input, alloc))(input)?;
    input = remain;

    let mut result = TableConstructor {
        arraylike_fields: Default::default(),
        indexed_fields: Default::default(),
    };

    let (mut arraylike_cursor, mut indexed_cursor) = if let Some(head) = maybe_head {
        match head {
            Field::Arraylike { expression } => (
                result
                    .arraylike_fields
                    .cursor_mut()
                    .alloc_insert_advance(alloc, expression),
                result.indexed_fields.cursor_mut(),
            ),
            Field::Named { name, expression } => (
                result.arraylike_fields.cursor_mut(),
                result
                    .indexed_fields
                    .cursor_mut()
                    .alloc_insert_advance(alloc, (Expression::String(name.into()), expression)),
            ),
            Field::Indexed { index, expression } => (
                result.arraylike_fields.cursor_mut(),
                result
                    .indexed_fields
                    .cursor_mut()
                    .alloc_insert_advance(alloc, (index, expression)),
            ),
        }
    } else {
        let (remain, _) = opt(one_of(",;"))(input)?;
        return Ok((remain, result));
    };

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, one_of(",;"), lua_whitespace0),
            |input| Field::parse(input, alloc),
        ))(input)?;
        input = remain;

        if let Some(next) = maybe_next {
            match next {
                Field::Arraylike { expression } => {
                    arraylike_cursor = arraylike_cursor.alloc_insert_advance(alloc, expression);
                }
                Field::Named { name, expression } => {
                    indexed_cursor = indexed_cursor
                        .alloc_insert_advance(alloc, (Expression::String(name.into()), expression));
                }
                Field::Indexed { index, expression } => {
                    indexed_cursor =
                        indexed_cursor.alloc_insert_advance(alloc, (index, expression));
                }
            }
        } else {
            break;
        };
    }

    let (remain, _) = opt(one_of(",;"))(input)?;

    Ok((remain, result))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::TableConstructor;
    use crate::{
        ast::expressions::{
            number::Number,
            Expression,
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            expressions::tables::Field,
            ASTAllocator,
            Parse,
            Span,
        },
    };

    #[test]
    pub fn parses_empty_table() -> anyhow::Result<()> {
        let src = "{}";

        let alloc = ASTAllocator::default();
        let (remain, result) = TableConstructor::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            TableConstructor {
                indexed_fields: Default::default(),
                arraylike_fields: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_empty_table_semicolon() -> anyhow::Result<()> {
        let src = "{;}";

        let alloc = ASTAllocator::default();
        let (remain, result) = TableConstructor::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            TableConstructor {
                indexed_fields: Default::default(),
                arraylike_fields: Default::default(),
            }
        );
        Ok(())
    }

    #[test]
    pub fn parses_arraylike_field() -> anyhow::Result<()> {
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
    pub fn parses_named_field() -> anyhow::Result<()> {
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
    pub fn parses_index_field() -> anyhow::Result<()> {
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
    pub fn parses_parses_table_mixed() -> anyhow::Result<()> {
        let src = "{10, [11] = 12, a = 13; 14}";

        let alloc = ASTAllocator::default();
        let (remain, result) = TableConstructor::parse(Span::new(src.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            result,
            TableConstructor {
                arraylike_fields: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(10))),
                    ListNode::new(Expression::Number(Number::Integer(14))),
                ]),
                indexed_fields: List::from_slice(&mut [
                    ListNode::new((
                        Expression::Number(Number::Integer(11)),
                        Expression::Number(Number::Integer(12))
                    )),
                    ListNode::new((
                        Expression::String("a".into()),
                        Expression::Number(Number::Integer(13))
                    ))
                ]),
            }
        );

        Ok(())
    }
}
