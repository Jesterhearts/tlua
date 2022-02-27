use crate::{
    expressions::function_defs::{
        FnBody,
        FnName,
    },
    identifiers::Ident,
    lexer::Token,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub enum FnDecl<'chunk> {
    Function {
        name: FnName<'chunk>,
        body: FnBody<'chunk>,
    },
    Local {
        name: Ident,
        body: FnBody<'chunk>,
    },
}

impl<'chunk> FnDecl<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let is_local = lexer.next_if_eq(Token::KWlocal);

        lexer.expecting_token(Token::KWfunction).map_err(|e| {
            if let Some(is_local) = is_local {
                lexer.reset(is_local)
            };
            e
        })?;

        if is_local.is_some() {
            Ok(Self::Local {
                name: Ident::parse(lexer, alloc).mark_unrecoverable()?,
                body: FnBody::parse(lexer, alloc).mark_unrecoverable()?,
            })
        } else {
            Ok(Self::Function {
                name: FnName::parse(lexer, alloc).mark_unrecoverable()?,
                body: FnBody::parse(lexer, alloc).mark_unrecoverable()?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expressions::function_defs::{
            FnBody,
            FnName,
            FnParams,
        },
        final_parser,
        identifiers::Ident,
        list::{
            List,
            ListNode,
        },
        statement::fn_decl::FnDecl,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_local_fn_def() -> anyhow::Result<()> {
        let src = "local function foo() end";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();

        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnDecl::parse)?;

        assert_eq!(
            result,
            FnDecl::Local {
                name: Ident(0),
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_def() -> anyhow::Result<()> {
        let src = "function foo() end";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();

        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnDecl::parse)?;

        assert_eq!(
            result,
            FnDecl::Function {
                name: FnName {
                    path: List::new(&mut ListNode::new(Ident(0))),
                    method: None
                },
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }
        );

        Ok(())
    }

    #[test]
    pub fn parses_method_fn_def() -> anyhow::Result<()> {
        let src = "function foo.bar:baz() end";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();

        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => FnDecl::parse)?;

        assert_eq!(
            result,
            FnDecl::Function {
                name: FnName {
                    path: List::from_slice(
                        &mut [ListNode::new(Ident(0)), ListNode::new(Ident(1)),]
                    ),
                    method: Some(Ident(2))
                },
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }
        );

        Ok(())
    }
}
