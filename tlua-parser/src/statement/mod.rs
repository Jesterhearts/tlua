use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char as token,
    combinator::{
        cut,
        map,
        value,
    },
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
    },
};

use crate::{
    block::Block,
    expressions::build_expression_list1,
    identifiers::{
        keyword,
        Ident,
    },
    lua_whitespace0,
    lua_whitespace1,
    prefix_expression::{
        FnCallPrefixExpression,
        PrefixExpression,
    },
    statement::{
        assignment::Assignment,
        fn_decl::FnDecl,
        for_loop::ForLoop,
        foreach_loop::ForEachLoop,
        if_statement::If,
        repeat_loop::RepeatLoop,
        variables::{
            varlist1,
            LocalVarList,
        },
        while_loop::WhileLoop,
    },
    ASTAllocator,
    ParseResult,
    Span,
    SyntaxError,
};

pub mod assignment;
pub mod fn_decl;
pub mod for_loop;
pub mod foreach_loop;
pub mod if_statement;
pub mod repeat_loop;
pub mod variables;
pub mod while_loop;

#[derive(Debug, PartialEq)]
pub struct Empty;

#[derive(Debug, PartialEq)]
pub struct Break;

#[derive(Debug, PartialEq)]
pub struct Label(pub Ident);

#[derive(Debug, PartialEq)]
pub struct Goto(pub Ident);

#[derive(Debug, PartialEq)]
pub enum Statement<'chunk> {
    Empty(Empty),
    Assignment(&'chunk Assignment<'chunk>),
    Call(&'chunk FnCallPrefixExpression<'chunk>),
    // TODO(lang-5.4): Scoping & matching rules.
    Label(Label),
    Break(Break),
    // TODO(lang-5.4): Scoping & matching rules.
    Goto(Goto),
    Do(&'chunk Block<'chunk>),
    While(&'chunk WhileLoop<'chunk>),
    Repeat(&'chunk RepeatLoop<'chunk>),
    If(&'chunk If<'chunk>),
    For(&'chunk ForLoop<'chunk>),
    ForEach(&'chunk ForEachLoop<'chunk>),
    FnDecl(&'chunk FnDecl<'chunk>),
    LocalVarList(&'chunk LocalVarList<'chunk>),
}

impl<'chunk> Statement<'chunk> {
    pub(crate) fn parser(
        alloc: &'chunk ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, Statement<'chunk>> {
        |input| {
            alt((
                map(token(';'), |_| Self::Empty(Empty)),
                map(
                    preceded(
                        pair(tag("::"), lua_whitespace0),
                        cut(terminated(
                            Ident::parser(alloc),
                            pair(lua_whitespace0, tag("::")),
                        )),
                    ),
                    |ident| Self::Label(Label(ident)),
                ),
                map(keyword("break"), |_| Self::Break(Break)),
                map(
                    preceded(
                        pair(keyword("goto"), lua_whitespace1),
                        cut(Ident::parser(alloc)),
                    ),
                    |ident| Self::Goto(Goto(ident)),
                ),
                delimited(
                    pair(keyword("do"), lua_whitespace0),
                    map(Block::parser(alloc), |block| Self::Do(alloc.alloc(block))),
                    pair(lua_whitespace0, keyword("end")),
                ),
                map(WhileLoop::parser(alloc), |stat| {
                    Self::While(alloc.alloc(stat))
                }),
                map(RepeatLoop::parser(alloc), |stat| {
                    Self::Repeat(alloc.alloc(stat))
                }),
                map(If::parser(alloc), |stat| Self::If(alloc.alloc(stat))),
                map(ForLoop::parser(alloc), |stat| Self::For(alloc.alloc(stat))),
                map(ForEachLoop::parser(alloc), |stat| {
                    Self::ForEach(alloc.alloc(stat))
                }),
                map(FnDecl::parser(alloc), |stat| {
                    Self::FnDecl(alloc.alloc(stat))
                }),
                map(LocalVarList::parser(alloc), |stat| {
                    Self::LocalVarList(alloc.alloc(stat))
                }),
                |input| parse_assignment_or_call(input, alloc),
            ))(input)
        }
    }
}

fn parse_assignment_or_call<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Statement<'chunk>> {
    let (remain, expr) = PrefixExpression::parser(alloc)(input)?;
    input = remain;

    match expr {
        PrefixExpression::FnCall(call) => Ok((remain, Statement::Call(alloc.alloc(call)))),
        PrefixExpression::Variable(var) => {
            let (input, varlist) = varlist1(input, var, alloc)?;

            let (input, expressions) = preceded(
                value((), delimited(lua_whitespace0, token('='), lua_whitespace0)),
                build_expression_list1(alloc),
            )(input)?;

            Ok((
                input,
                Statement::Assignment(alloc.alloc(Assignment {
                    varlist,
                    expressions,
                })),
            ))
        }
        PrefixExpression::Parenthesized(_) => {
            Err(nom::Err::Error(nom_supreme::error::ErrorTree::Base {
                location: input,
                kind: nom_supreme::error::BaseErrorKind::External(Box::new(
                    SyntaxError::ExpectedVarOrCall,
                )),
            }))
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::Statement;
    use crate::{
        block::Block,
        expressions::{
            number::Number,
            Expression,
        },
        final_parser,
        list::{
            List,
            ListNode,
        },
        prefix_expression::VarPrefixExpression,
        statement::{
            assignment::Assignment,
            Break,
            Empty,
            Goto,
            Label,
        },
        ASTAllocator,
        Span,
    };

    #[test]
    fn parses_empty_stat() -> anyhow::Result<()> {
        let src = ";";

        let alloc = ASTAllocator::default();
        let stat = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(stat, Statement::Empty(Empty));

        Ok(())
    }

    #[test]
    fn parses_break_stat() -> anyhow::Result<()> {
        let src = "break";

        let alloc = ASTAllocator::default();
        let stat = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(stat, Statement::Break(Break));

        Ok(())
    }

    #[test]
    fn parses_label_stat() -> anyhow::Result<()> {
        let src = "::foo::";

        let alloc = ASTAllocator::default();
        let stat = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(stat, Statement::Label(Label("foo".into())));

        Ok(())
    }

    #[test]
    fn parses_goto_stat() -> anyhow::Result<()> {
        let src = "goto foo";

        let alloc = ASTAllocator::default();
        let stat = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(stat, Statement::Goto(Goto("foo".into())));

        Ok(())
    }

    #[test]
    fn parses_goto_handles_name() -> anyhow::Result<()> {
        let src = "goto gotofoo";

        let alloc = ASTAllocator::default();
        let stat = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(stat, Statement::Goto(Goto("gotofoo".into())));

        Ok(())
    }

    #[test]
    fn parses_do_stat() -> anyhow::Result<()> {
        let src = "do end";

        let alloc = ASTAllocator::default();
        let stat = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(
            stat,
            Statement::Do(&Block {
                statements: Default::default(),
                ret: None
            })
        );

        Ok(())
    }

    #[test]
    fn parses_single() -> anyhow::Result<()> {
        let src = "a = 10";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(
            result,
            Statement::Assignment(&Assignment {
                varlist: List::new(&mut ListNode::new(VarPrefixExpression::Name("a".into()))),
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10)))),
            })
        );

        Ok(())
    }

    #[test]
    fn parses_multi() -> anyhow::Result<()> {
        let src = "a ,b, c , d = 10, 11, 12 , 13";

        let alloc = ASTAllocator::default();
        let result = final_parser!(Span::new(src.as_bytes()) => Statement::parser(&alloc))?;

        assert_eq!(
            result,
            Statement::Assignment(&Assignment {
                varlist: List::from_slice(&mut [
                    ListNode::new(VarPrefixExpression::Name("a".into())),
                    ListNode::new(VarPrefixExpression::Name("b".into())),
                    ListNode::new(VarPrefixExpression::Name("c".into())),
                    ListNode::new(VarPrefixExpression::Name("d".into())),
                ]),
                expressions: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(10))),
                    ListNode::new(Expression::Number(Number::Integer(11))),
                    ListNode::new(Expression::Number(Number::Integer(12))),
                    ListNode::new(Expression::Number(Number::Integer(13))),
                ]),
            })
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::Comparison;

    use super::Statement;

    #[test]
    fn sizeof_statement() {
        let left = std::mem::size_of::<Statement>();
        let right = std::mem::size_of::<usize>() * 4;
        if left > right {
            panic!(
                "assertion failed: `(left <= right)`\
                        \n\
                        \n{}:\
                        \n\
                        \n",
                Comparison::new(&left, &right)
            );
        }
    }
}
