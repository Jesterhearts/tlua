use crate::{
    block::Block,
    combinators::{
        parse_list0_split_tail,
        parse_list1_split_tail_or,
    },
    expressions::{
        function_defs::{
            FnBody,
            FnName,
        },
        Expression,
    },
    identifiers::Ident,
    lexer::Token,
    prefix_expression::{
        FnCallPrefixExpression,
        HeadAtom,
        PrefixAtom,
        VarPrefixExpression,
    },
    statement::{
        assignment::Assignment,
        fn_decl::FnDecl,
        for_loop::ForLoop,
        foreach_loop::ForEachLoop,
        if_statement::If,
        repeat_loop::RepeatLoop,
        variables::LocalVarList,
        while_loop::WhileLoop,
    },
    token_subset,
    ASTAllocator,
    ParseError,
    PeekableLexer,
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

impl Label {
    pub(crate) fn parse_remaining(
        lexer: &mut PeekableLexer,
        alloc: &ASTAllocator,
    ) -> Result<Self, ParseError> {
        Ident::parse(lexer, alloc).and_then(|ident| {
            lexer
                .expecting_token(Token::DoubleColon)
                .map(|_| Self(ident))
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Goto(pub Ident);

#[derive(Debug, PartialEq)]
pub enum Statement<'chunk> {
    Empty(Empty),
    Label(Label),
    Break(Break),
    Goto(Goto),
    Do(&'chunk Block<'chunk>),
    While(&'chunk WhileLoop<'chunk>),
    Repeat(&'chunk RepeatLoop<'chunk>),
    If(&'chunk If<'chunk>),
    Assignment(&'chunk Assignment<'chunk>),
    Call(&'chunk FnCallPrefixExpression<'chunk>),
    For(&'chunk ForLoop<'chunk>),
    ForEach(&'chunk ForEachLoop<'chunk>),
    FnDecl(&'chunk FnDecl<'chunk>),
    LocalVarList(&'chunk LocalVarList<'chunk>),
}

token_subset! {
    pub(crate) StatementToken {
        Token::Semicolon,
        Token::KWbreak,
        Token::DoubleColon,
        Token::KWdo,
        Token::KWgoto,
        Token::KWwhile,
        Token::KWrepeat,
        Token::KWif,
        Token::KWfunction,
        Token::KWlocal,
        Token::Ident,
        Token::LParen,
        Token::KWfor,
        Error(SyntaxError::ExpectedStatement)
    }
}

impl<'chunk> Statement<'chunk> {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Option<Self>, ParseError> {
        let token = if let Some(token) = StatementToken::next(lexer) {
            token
        } else {
            return Ok(None);
        };

        Ok(Some(match token.as_ref() {
            StatementToken::Semicolon => Self::Empty(Empty),
            StatementToken::KWbreak => Self::Break(Break),
            StatementToken::DoubleColon => Self::Label(Label::parse_remaining(lexer, alloc)?),
            StatementToken::KWgoto => Self::Goto(Goto(Ident::parse(lexer, alloc)?)),
            StatementToken::KWdo => Self::Do(alloc.alloc(Block::parse_with_end(lexer, alloc)?)),
            StatementToken::KWwhile => {
                Self::While(alloc.alloc(WhileLoop::parse_remaining(lexer, alloc)?))
            }
            StatementToken::KWrepeat => {
                Self::Repeat(alloc.alloc(RepeatLoop::parse_remaining(lexer, alloc)?))
            }
            StatementToken::KWif => Self::If(alloc.alloc(If::parse_remaining(lexer, alloc)?)),
            StatementToken::KWfunction => Self::FnDecl(alloc.alloc(FnDecl::Function {
                name: FnName::parse(lexer, alloc)?,
                body: FnBody::parse(lexer, alloc)?,
            })),
            StatementToken::KWlocal => {
                token_subset! {
                    LocalDecl {
                        Token::KWfunction,
                        Token::Ident,
                        Error(SyntaxError::ExpectedToken2(Token::KWfunction, Token::Ident))
                    }
                }

                let token = LocalDecl::expect_next(lexer)?;
                match token.as_ref() {
                    LocalDecl::KWfunction => Self::FnDecl(alloc.alloc(FnDecl::Local {
                        name: Ident::parse(lexer, alloc)?,
                        body: FnBody::parse(lexer, alloc)?,
                    })),
                    LocalDecl::Ident => {
                        let head_ident = lexer.strings.add_ident(token.src);
                        Self::LocalVarList(
                            alloc.alloc(LocalVarList::parse_remaining(head_ident, lexer, alloc)?),
                        )
                    }
                }
            }
            StatementToken::Ident => {
                // This will always be valid since it can only expand to a prefix expression for
                // a variable or a function call.
                let head = lexer.strings.add_ident(token.src);
                match parse_list0_split_tail(lexer, alloc, PrefixAtom::try_parse)? {
                    Some((middle, tail)) => {
                        let head = HeadAtom::Name(head);
                        match tail {
                            PrefixAtom::Var(tail) => {
                                Self::Assignment(alloc.alloc(Assignment::parse_remaining(
                                    VarPrefixExpression::TableAccess {
                                        head,
                                        middle,
                                        last: alloc.alloc(tail),
                                    },
                                    lexer,
                                    alloc,
                                )?))
                            }
                            PrefixAtom::Function(tail) => Self::Call(
                                alloc.alloc(FnCallPrefixExpression::from((head, middle, tail))),
                            ),
                        }
                    }
                    None => Self::Assignment(alloc.alloc(Assignment::parse_remaining(
                        VarPrefixExpression::Name(head),
                        lexer,
                        alloc,
                    )?)),
                }
            }
            StatementToken::LParen => {
                // This is possibly invalid if it expands to a parenthesized expression and
                // doesn't turn out to be a variable path or function call.
                //
                // ```
                // Invalid:
                //  - `(foo) = bar`
                //           ^ invalid here, `(foo)` is an expression not a var or fncall.
                // Valid:
                //  - `(foo).a = bar`
                //  - `(foo)[a] = bar`
                //  - `(foo)(bar)` <--+ these are calls.
                //  - `(foo)"bar"` <--+
                //  - `(foo){bar}` <--+
                // ```
                let head = Expression::parse(lexer, alloc)
                    .and_then(|expr| lexer.expecting_token(Token::RParen).map(|_| expr))?;
                let head = HeadAtom::Parenthesized(alloc.alloc(head));
                let (middle, tail) = parse_list1_split_tail_or(
                    lexer,
                    alloc,
                    PrefixAtom::try_parse,
                    SyntaxError::ExpectedVarOrCall,
                )?;

                match tail {
                    PrefixAtom::Var(var) => {
                        let head = VarPrefixExpression::TableAccess {
                            head,
                            middle,
                            last: alloc.alloc(var),
                        };
                        Self::Assignment(
                            alloc.alloc(Assignment::parse_remaining(head, lexer, alloc)?),
                        )
                    }
                    PrefixAtom::Function(tail) => {
                        Self::Call(alloc.alloc(FnCallPrefixExpression::from((head, middle, tail))))
                    }
                }
            }
            StatementToken::KWfor => {
                // There are two different AST nodes which use the `for` keyword - the numeric
                // for loop and the generic for loop, so we need to parse further to
                // disambiguate.
                // We disambiguate by:
                // ```
                //  - `for ident`
                //     ^^^ ^^^^^ this is shared among all for loops.
                //  - `for ident =`
                //               ^ we know it's a numeric for loop here.
                //  - `for ident in`
                //               ^^ we know it's a generic for loop here.
                //  - `for ident, ident in`
                //              ^ we know it's a generic for loop here.
                // ```
                token_subset! {
                    ForNext {
                        Token::Equals,
                        Token::Comma,
                        Token::KWin,
                        Error(SyntaxError::ExpectedToken3(Token::Equals, Token::Comma, Token::KWin))
                    }
                };
                let head = Ident::parse(lexer, alloc)?;
                let token = lexer
                    .peek()
                    .filter(ForNext::matches)
                    .ok_or_else(|| ParseError::from_here(lexer, ForNext::ERROR))?;

                match token.as_ref() {
                    Token::Equals => {
                        Self::For(alloc.alloc(ForLoop::parse_remaining(head, lexer, alloc)?))
                    }
                    _ => Self::ForEach(
                        alloc.alloc(ForEachLoop::parse_remaining(head, lexer, alloc)?),
                    ),
                }
            }
        }))
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{
        assert_eq,
        Comparison,
    };

    use super::Statement;
    use crate::{
        block::Block,
        expressions::{
            function_defs::{
                FnBody,
                FnName,
                FnParams,
            },
            number::Number,
            Expression,
        },
        final_parser,
        identifiers::Ident,
        list::{
            List,
            ListNode,
        },
        prefix_expression::VarPrefixExpression,
        statement::{
            assignment::Assignment,
            fn_decl::FnDecl,
            for_loop::ForLoop,
            foreach_loop::ForEachLoop,
            if_statement::{
                ElseIf,
                If,
            },
            repeat_loop::RepeatLoop,
            variables::{
                Attribute,
                LocalVar,
                LocalVarList,
            },
            while_loop::WhileLoop,
            Break,
            Empty,
            Goto,
            Label,
        },
        ASTAllocator,
        StringTable,
    };

    #[test]
    fn parses_empty_stat() -> anyhow::Result<()> {
        let src = ";";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(stat, Some(Statement::Empty(Empty)));

        Ok(())
    }

    #[test]
    fn parses_break_stat() -> anyhow::Result<()> {
        let src = "break";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(stat, Some(Statement::Break(Break)));

        Ok(())
    }

    #[test]
    fn parses_label_stat() -> anyhow::Result<()> {
        let src = "::foo::";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(stat, Some(Statement::Label(Label(Ident(0)))));

        Ok(())
    }

    #[test]
    fn parses_goto_stat() -> anyhow::Result<()> {
        let src = "goto foo";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(stat, Some(Statement::Goto(Goto(Ident(0)))));

        Ok(())
    }

    #[test]
    fn parses_goto_handles_name() -> anyhow::Result<()> {
        let src = "goto gotofoo";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(stat, Some(Statement::Goto(Goto(Ident(0)))));

        Ok(())
    }

    #[test]
    fn parses_do_stat() -> anyhow::Result<()> {
        let src = "do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            stat,
            Some(Statement::Do(&Block {
                statements: Default::default(),
                ret: None
            }))
        );

        Ok(())
    }

    #[test]
    fn parses_single() -> anyhow::Result<()> {
        let src = "a = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::Assignment(&Assignment {
                varlist: List::new(&mut ListNode::new(VarPrefixExpression::Name(Ident(0)))),
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10)))),
            }))
        );

        Ok(())
    }

    #[test]
    fn parses_multi() -> anyhow::Result<()> {
        let src = "a ,b, c , d = 10, 11, 12 , 13";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::Assignment(&Assignment {
                varlist: List::from_slice(&mut [
                    ListNode::new(VarPrefixExpression::Name(Ident(0))),
                    ListNode::new(VarPrefixExpression::Name(Ident(1))),
                    ListNode::new(VarPrefixExpression::Name(Ident(2))),
                    ListNode::new(VarPrefixExpression::Name(Ident(3))),
                ]),
                expressions: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(10))),
                    ListNode::new(Expression::Number(Number::Integer(11))),
                    ListNode::new(Expression::Number(Number::Integer(12))),
                    ListNode::new(Expression::Number(Number::Integer(13))),
                ]),
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_local_fn_def() -> anyhow::Result<()> {
        let src = "local function foo() end";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();

        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::FnDecl(&FnDecl::Local {
                name: Ident(0),
                body: FnBody {
                    params: FnParams {
                        named_params: Default::default(),
                        varargs: false
                    },
                    body: Default::default()
                }
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_fn_def() -> anyhow::Result<()> {
        let src = "function foo() end";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();

        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::FnDecl(&FnDecl::Function {
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
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_method_fn_def() -> anyhow::Result<()> {
        let src = "function foo.bar:baz() end";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();

        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::FnDecl(&FnDecl::Function {
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
            }))
        );

        Ok(())
    }

    #[test]
    fn parses_local() -> anyhow::Result<()> {
        let local = "local foo";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            decl,
            Some(Statement::LocalVarList(&LocalVarList {
                vars: List::new(&mut ListNode::new(LocalVar {
                    name: Ident(0),
                    attribute: None
                })),
                initializers: Default::default(),
            }))
        );

        Ok(())
    }

    #[test]
    fn parses_local_namelist() -> anyhow::Result<()> {
        let local = "local foo,bar";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            decl,
            Some(Statement::LocalVarList(&LocalVarList {
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
            }))
        );

        Ok(())
    }

    #[test]
    fn parses_local_with_attrib() -> anyhow::Result<()> {
        let local = "local foo<const>, bar<close>";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            decl,
            Some(Statement::LocalVarList(&LocalVarList {
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
            }))
        );

        Ok(())
    }

    #[test]
    fn parses_local_init() -> anyhow::Result<()> {
        let local = "local foo,bar = 10";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let decl = final_parser!((local.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            decl,
            Some(Statement::LocalVarList(&LocalVarList {
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
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_foreach() -> anyhow::Result<()> {
        let src = "for a,b,c,d in 1,2,3,4 do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::ForEach(&ForEachLoop {
                vars: List::from_slice(&mut [
                    ListNode::new(Ident(0)),
                    ListNode::new(Ident(1)),
                    ListNode::new(Ident(2)),
                    ListNode::new(Ident(3)),
                ]),
                expressions: List::from_slice(&mut [
                    ListNode::new(Expression::Number(Number::Integer(1))),
                    ListNode::new(Expression::Number(Number::Integer(2))),
                    ListNode::new(Expression::Number(Number::Integer(3))),
                    ListNode::new(Expression::Number(Number::Integer(4))),
                ]),
                body: Default::default()
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_for() -> anyhow::Result<()> {
        let src = "for a = 0, 10 do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::For(&ForLoop {
                var: Ident(0),
                init: Expression::Number(Number::Integer(0)),
                condition: Expression::Number(Number::Integer(10)),
                increment: None,
                body: Default::default()
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_if() -> anyhow::Result<()> {
        let src = "if true then end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::If(&If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: Default::default(),
                else_final: None,
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_if_else() -> anyhow::Result<()> {
        let src = "if true then else end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::If(&If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: Default::default(),
                else_final: Some(Default::default())
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_if_elseif_else() -> anyhow::Result<()> {
        let src = "if true then elseif true then else end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::If(&If {
                cond: Expression::Bool(true),
                body: Default::default(),
                elif: List::new(&mut ListNode::new(ElseIf {
                    cond: Expression::Bool(true),
                    body: Default::default(),
                })),
                else_final: Some(Default::default())
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_repeat() -> anyhow::Result<()> {
        let src = "repeat until true";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::Repeat(&RepeatLoop {
                body: Default::default(),
                terminator: Expression::Bool(true)
            }))
        );

        Ok(())
    }

    #[test]
    pub fn parses_while() -> anyhow::Result<()> {
        let src = "while true do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::try_parse)?;

        assert_eq!(
            result,
            Some(Statement::While(&WhileLoop {
                cond: Expression::Bool(true),
                body: Default::default()
            }))
        );

        Ok(())
    }

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
