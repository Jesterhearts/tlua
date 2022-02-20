use nom::{
    bytes::complete::tag,
    combinator::{
        map,
        map_res,
        opt,
    },
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
    },
};

use crate::{
    expressions::{
        expression_list1,
        Expression,
    },
    identifiers::{
        parse_identifier,
        Ident,
    },
    list::List,
    lua_whitespace0,
    lua_whitespace1,
    prefix_expression::{
        PrefixExpression,
        VarPrefixExpression,
    },
    ASTAllocator,
    Parse,
    ParseResult,
    Span,
    SyntaxError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Attribute {
    Const,
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVar {
    pub name: Ident,
    pub attribute: Option<Attribute>,
}

#[derive(Debug, PartialEq)]
pub struct LocalVarList<'chunk> {
    pub vars: List<'chunk, LocalVar>,
    pub initializers: List<'chunk, Expression<'chunk>>,
}

impl<'chunk> Parse<'chunk> for LocalVar {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        map_res(
            pair(
                terminated(|input| parse_identifier(input, alloc), lua_whitespace0),
                opt(delimited(
                    pair(tag("<"), lua_whitespace0),
                    |input| parse_identifier(input, alloc),
                    pair(lua_whitespace0, tag(">")),
                )),
            ),
            |(name, attribute)| {
                let attribute = match attribute {
                    None => None,
                    Some(attribute) => Some(match &*attribute {
                        b"const" => Attribute::Const,
                        b"close" => Attribute::Close,
                        _ => return Err(SyntaxError::InvalidAttribute),
                    }),
                };

                Ok(Self { name, attribute })
            },
        )(input)
    }
}

impl<'chunk> Parse<'chunk> for LocalVarList<'chunk> {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        map(
            preceded(
                pair(tag("local"), lua_whitespace1),
                pair(
                    |input| local_varlist1(input, alloc),
                    opt(preceded(
                        delimited(lua_whitespace0, tag("="), lua_whitespace0),
                        |input| expression_list1(input, alloc),
                    )),
                ),
            ),
            |(vars, initializers)| Self {
                vars,
                initializers: initializers.unwrap_or_default(),
            },
        )(input)
    }
}

pub fn local_varlist1<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, LocalVar>> {
    let (remain, head) = LocalVar::parse(input, alloc)?;
    input = remain;

    let mut locals = List::default();
    let current = locals.cursor_mut();
    let mut current = current.alloc_insert_advance(alloc, head);

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, tag(","), lua_whitespace0),
            |input| LocalVar::parse(input, alloc),
        ))(input)?;
        input = remain;

        current = if let Some(next) = maybe_next {
            current.alloc_insert_advance(alloc, next)
        } else {
            return Ok((input, locals));
        }
    }
}

pub fn varlist1<'src, 'chunk>(
    mut input: Span<'src>,
    head: VarPrefixExpression<'chunk>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, VarPrefixExpression<'chunk>>> {
    let mut list = List::default();
    let mut current = list.cursor_mut().alloc_insert_advance(alloc, head);

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, tag(","), lua_whitespace0),
            map_res(
                |input| PrefixExpression::parse(input, alloc),
                |expr| match expr {
                    PrefixExpression::Variable(var) => Ok(var),
                    PrefixExpression::FnCall(_) | PrefixExpression::Parenthesized(_) => {
                        Err(SyntaxError::ExpectedVariable)
                    }
                },
            ),
        ))(input)?;
        input = remain;

        current = if let Some(next) = maybe_next {
            current.alloc_insert_advance(alloc, next)
        } else {
            return Ok((input, list));
        };
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        expressions::{
            number::Number,
            Expression,
        },
        list::{
            List,
            ListNode,
        },
        statement::variables::{
            ASTAllocator,
            Attribute,
            LocalVar,
            LocalVarList,
            Parse,
            Span,
        },
    };

    #[test]
    pub fn parses_local() -> anyhow::Result<()> {
        let local = "local foo";

        let alloc = ASTAllocator::default();
        let (remain, decl) = LocalVarList::parse(Span::new(local.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            decl,
            LocalVarList {
                vars: List::new(&mut ListNode::new(LocalVar {
                    name: "foo".into(),
                    attribute: None
                })),
                initializers: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_local_namelist() -> anyhow::Result<()> {
        let local = "local foo,bar";

        let alloc = ASTAllocator::default();
        let (remain, decl) = LocalVarList::parse(Span::new(local.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            decl,
            LocalVarList {
                vars: List::from_slice(&mut [
                    ListNode::new(LocalVar {
                        name: "foo".into(),
                        attribute: None
                    }),
                    ListNode::new(LocalVar {
                        name: "bar".into(),
                        attribute: None
                    })
                ]),
                initializers: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_local_with_attrib() -> anyhow::Result<()> {
        let local = "local foo<const>, bar<close>";

        let alloc = ASTAllocator::default();
        let (remain, decl) = LocalVarList::parse(Span::new(local.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            decl,
            LocalVarList {
                vars: List::from_slice(&mut [
                    ListNode::new(LocalVar {
                        name: "foo".into(),
                        attribute: Some(Attribute::Const)
                    }),
                    ListNode::new(LocalVar {
                        name: "bar".into(),
                        attribute: Some(Attribute::Close)
                    }),
                ]),
                initializers: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_local_init() -> anyhow::Result<()> {
        let local = "local foo,bar = 10";

        let alloc = ASTAllocator::default();
        let (remain, decl) = LocalVarList::parse(Span::new(local.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            decl,
            LocalVarList {
                vars: List::from_slice(&mut [
                    ListNode::new(LocalVar {
                        name: "foo".into(),
                        attribute: None
                    }),
                    ListNode::new(LocalVar {
                        name: "bar".into(),
                        attribute: None
                    })
                ]),
                initializers: List::new(&mut ListNode::new(Expression::Number(Number::Integer(
                    10
                )))),
            }
        );

        Ok(())
    }
}
