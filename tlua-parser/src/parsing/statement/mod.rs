use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char as token,
    combinator::{
        map,
        value,
    },
    sequence::{
        delimited,
        pair,
        preceded,
    },
};
use nom_supreme::ParserExt;
use tracing::instrument;

use crate::{
    ast::{
        block::Block,
        prefix_expression::PrefixExpression,
        statement::{
            assignment::Assignment,
            fn_decl::FnDecl,
            for_loop::ForLoop,
            foreach_loop::ForEachLoop,
            if_statement::If,
            repeat_loop::RepeatLoop,
            variables::LocalVarList,
            while_loop::WhileLoop,
            Break,
            Empty,
            Goto,
            Label,
            Statement,
        },
    },
    parsing::{
        expressions::expression_list1,
        identifiers::parse_identifier,
        lua_whitespace0,
        lua_whitespace1,
        ASTAllocator,
        Parse,
        ParseResult,
        Span,
        SyntaxError,
    },
};

pub mod fn_decl;
pub mod for_loop;
pub mod foreach_loop;
pub mod if_statement;
pub mod repeat_loop;
pub mod variables;
pub mod while_loop;

use self::variables::varlist1;

impl<'chunk> Parse<'chunk> for Statement<'chunk> {
    #[instrument(level = "trace", name = "statement", skip(input, alloc))]
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self> {
        alt((
            map(token(';'), |_| Self::Empty(Empty)).context("empty statement"),
            map(
                delimited(
                    pair(tag("::"), lua_whitespace0),
                    |input| parse_identifier(input, alloc),
                    pair(lua_whitespace0, tag("::")),
                ),
                |ident| Self::Label(Label(ident)),
            )
            .context("label statement"),
            map(tag("break"), |_| Self::Break(Break)).context("break statement"),
            map(
                preceded(pair(tag("goto"), lua_whitespace1), |input| {
                    parse_identifier(input, alloc)
                }),
                |ident| Self::Goto(Goto(ident)),
            )
            .context("goto statement"),
            delimited(
                pair(tag("do"), lua_whitespace0),
                map(
                    |input| Block::parse(input, alloc),
                    |block| Self::Do(alloc.alloc(block)),
                ),
                pair(lua_whitespace0, tag("end")),
            )
            .context("do statement"),
            map(
                |input| WhileLoop::parse(input, alloc),
                |stat| Self::While(alloc.alloc(stat)),
            )
            .context("while statement"),
            map(
                |input| RepeatLoop::parse(input, alloc),
                |stat| Self::Repeat(alloc.alloc(stat)),
            )
            .context("repeat statement"),
            map(
                |input| If::parse(input, alloc),
                |stat| Self::If(alloc.alloc(stat)),
            )
            .context("if statement"),
            map(
                |input| ForLoop::parse(input, alloc),
                |stat| Self::For(alloc.alloc(stat)),
            )
            .context("for statement"),
            map(
                |input| ForEachLoop::parse(input, alloc),
                |stat| Self::ForEach(alloc.alloc(stat)),
            )
            .context("foreach statement"),
            map(
                |input| FnDecl::parse(input, alloc),
                |stat| Self::FnDecl(alloc.alloc(stat)),
            )
            .context("function declaration"),
            map(
                |input| LocalVarList::parse(input, alloc),
                |stat| Self::LocalVarList(alloc.alloc(stat)),
            )
            .context("local variable declaration"),
            |input| parse_assignment_or_call(input, alloc),
        ))(input)
    }
}

#[instrument(level = "trace", name = "assign_or_call", skip(input, alloc))]
fn parse_assignment_or_call<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Statement<'chunk>> {
    let (remain, expr) = PrefixExpression::parse(input, alloc)?;
    input = remain;

    match expr {
        PrefixExpression::FnCall(call) => Ok((remain, Statement::Call(alloc.alloc(call)))),
        PrefixExpression::Variable(var) => {
            let (input, varlist) = varlist1(input, var, alloc)?;

            let (input, expressions) = preceded(
                value((), delimited(lua_whitespace0, tag("="), lua_whitespace0)),
                |input| expression_list1(input, alloc),
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
    use nom_supreme::final_parser::final_parser;
    use pretty_assertions::assert_eq;

    use super::Statement;
    use crate::{
        ast::{
            block::Block,
            expressions::{
                number::Number,
                Expression,
            },
            prefix_expression::VarPrefixExpression,
            statement::{
                assignment::Assignment,
                Break,
                Empty,
                Goto,
                Label,
            },
        },
        list::{
            List,
            ListNode,
        },
        parsing::{
            ASTAllocator,
            InternalLuaParseError,
            LuaParseError,
            Parse,
            Span,
        },
    };

    #[test]
    fn parses_empty_stat() -> anyhow::Result<()> {
        let src = ";";

        let alloc = ASTAllocator::default();
        let stat = final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(stat, Statement::Empty(Empty));

        Ok(())
    }

    #[test]
    fn parses_break_stat() -> anyhow::Result<()> {
        let src = "break";

        let alloc = ASTAllocator::default();
        let stat = final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(stat, Statement::Break(Break));

        Ok(())
    }

    #[test]
    fn parses_label_stat() -> anyhow::Result<()> {
        let src = "::foo::";

        let alloc = ASTAllocator::default();
        let stat = final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(stat, Statement::Label(Label("foo".into())));

        Ok(())
    }

    #[test]
    fn parses_goto_stat() -> anyhow::Result<()> {
        let src = "goto foo";

        let alloc = ASTAllocator::default();
        let stat = final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(stat, Statement::Goto(Goto("foo".into())));

        Ok(())
    }

    #[test]
    fn parses_goto_handles_name() -> anyhow::Result<()> {
        let src = "goto gotofoo";

        let alloc = ASTAllocator::default();
        let stat = final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

        assert_eq!(stat, Statement::Goto(Goto("gotofoo".into())));

        Ok(())
    }

    #[test]
    fn parses_do_stat() -> anyhow::Result<()> {
        let src = "do end";

        let alloc = ASTAllocator::default();
        let stat = final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
            .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

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
        let result =
            final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

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
        let result =
            final_parser(|input| Statement::parse(input, &alloc))(Span::new(src.as_bytes()))
                .map_err(|e: InternalLuaParseError| e.map_locations(LuaParseError::from))?;

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
