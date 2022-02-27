use crate::{
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    list::List,
    parse_separated_list1,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
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

impl LocalVar {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &ASTAllocator,
    ) -> Result<Self, ParseError> {
        let name = Ident::parse(lexer, alloc)?;
        let attribute = if lexer.next_if_eq(Token::LeftAngle).is_some() {
            let ident = lexer.expecting_token(Token::Ident).mark_unrecoverable()?;

            let attrib = match ident.src {
                b"const" => Attribute::Const,
                b"close" => Attribute::Close,
                _ => {
                    return Err(ParseError::unrecoverable_from_here(
                        lexer,
                        SyntaxError::InvalidAttribute,
                    ));
                }
            };
            lexer
                .expecting_token(Token::RightAngle)
                .mark_unrecoverable()?;
            Some(attrib)
        } else {
            None
        };

        Ok(Self { name, attribute })
    }
}

impl<'chunk> LocalVarList<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let local_token = lexer.expecting_token(Token::KWlocal)?;

        let vars = parse_separated_list1(lexer, alloc, LocalVar::parse, |token| {
            *token == Token::Comma
        })
        .reset_on_err(lexer, local_token)?;

        let initializers = if lexer.next_if_eq(Token::Equals).is_some() {
            Expression::parse_list1(lexer, alloc).mark_unrecoverable()?
        } else {
            Default::default()
        };

        Ok(Self { vars, initializers })
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
        final_parser,
        identifiers::Ident,
        list::{
            List,
            ListNode,
        },
        statement::variables::{
            ASTAllocator,
            Attribute,
            LocalVar,
            LocalVarList,
        },
        StringTable,
    };

    #[test]
    fn parses_local() -> anyhow::Result<()> {
        let local = "local foo";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => LocalVarList::parse)?;

        assert_eq!(
            decl,
            LocalVarList {
                vars: List::new(&mut ListNode::new(LocalVar {
                    name: Ident(0),
                    attribute: None
                })),
                initializers: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn parses_local_namelist() -> anyhow::Result<()> {
        let local = "local foo,bar";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => LocalVarList::parse)?;

        assert_eq!(
            decl,
            LocalVarList {
                vars: List::from_slice(&mut [
                    ListNode::new(LocalVar {
                        name: Ident(0),
                        attribute: None
                    }),
                    ListNode::new(LocalVar {
                        name: Ident(1),
                        attribute: None
                    })
                ]),
                initializers: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn parses_local_with_attrib() -> anyhow::Result<()> {
        let local = "local foo<const>, bar<close>";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => LocalVarList::parse)?;

        assert_eq!(
            decl,
            LocalVarList {
                vars: List::from_slice(&mut [
                    ListNode::new(LocalVar {
                        name: Ident(0),
                        attribute: Some(Attribute::Const)
                    }),
                    ListNode::new(LocalVar {
                        name: Ident(1),
                        attribute: Some(Attribute::Close)
                    }),
                ]),
                initializers: Default::default(),
            }
        );

        Ok(())
    }

    #[test]
    fn parses_local_init() -> anyhow::Result<()> {
        let local = "local foo,bar = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => LocalVarList::parse)?;

        assert_eq!(
            decl,
            LocalVarList {
                vars: List::from_slice(&mut [
                    ListNode::new(LocalVar {
                        name: Ident(0),
                        attribute: None
                    }),
                    ListNode::new(LocalVar {
                        name: Ident(1),
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
