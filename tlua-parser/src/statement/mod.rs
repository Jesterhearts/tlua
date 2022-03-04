use crate::{
    block::Block,
    identifiers::Ident,
    lexer::Token,
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
        variables::LocalVarList,
        while_loop::WhileLoop,
    },
    ASTAllocator,
    ParseError,
    ParseErrorExt,
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

impl Empty {
    pub(crate) fn parse(lexer: &mut PeekableLexer, _: &ASTAllocator) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::Semicolon).map(|_| Self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Break;

impl Break {
    pub(crate) fn parse(lexer: &mut PeekableLexer, _: &ASTAllocator) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::KWbreak).map(|_| Self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Label(pub Ident);

impl Label {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::DoubleColon)?;

        let label = Ident::parse(lexer, alloc).mark_unrecoverable().map(Self)?;
        lexer
            .expecting_token(Token::DoubleColon)
            .mark_unrecoverable()?;

        Ok(label)
    }
}

#[derive(Debug, PartialEq)]
pub struct Goto(pub Ident);

impl Goto {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &ASTAllocator,
    ) -> Result<Self, ParseError> {
        lexer.expecting_token(Token::KWgoto)?;

        Ident::parse(lexer, alloc).mark_unrecoverable().map(Self)
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement<'chunk> {
    Empty(Empty),
    Assignment(&'chunk Assignment<'chunk>),
    Call(&'chunk FnCallPrefixExpression<'chunk>),
    Label(Label),
    Break(Break),
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
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        { Empty::parse(lexer, alloc).map(Self::Empty) }
            .recover_with(|| Break::parse(lexer, alloc).map(Self::Break))
            .recover_with(|| Label::parse(lexer, alloc).map(Self::Label))
            .recover_with(|| Goto::parse(lexer, alloc).map(Self::Goto))
            .recover_with(|| {
                WhileLoop::parse(lexer, alloc).map(|stat| Self::While(alloc.alloc(stat)))
            })
            .recover_with(|| {
                RepeatLoop::parse(lexer, alloc).map(|stat| Self::Repeat(alloc.alloc(stat)))
            })
            .recover_with(|| If::parse(lexer, alloc).map(|stat| Self::If(alloc.alloc(stat))))
            .recover_with(|| {
                FnDecl::parse(lexer, alloc).map(|stat| Self::FnDecl(alloc.alloc(stat)))
            })
            .recover_with(|| {
                LocalVarList::parse(lexer, alloc).map(|stat| Self::LocalVarList(alloc.alloc(stat)))
            })
            .recover_with(|| ForLoop::parse(lexer, alloc).map(|stat| Self::For(alloc.alloc(stat))))
            .recover_with(|| {
                ForEachLoop::parse(lexer, alloc).map(|stat| Self::ForEach(alloc.alloc(stat)))
            })
            .recover_with(|| {
                Block::parse_do(lexer, alloc).map(|block| Self::Do(alloc.alloc(block)))
            })
            .recover_with(|| parse_assignment_or_call(lexer, alloc))
            .ok_or_else(|| ParseError::recoverable_from_here(lexer, SyntaxError::ExpectedStatement))
    }
}

fn parse_assignment_or_call<'chunk>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
) -> Result<Statement<'chunk>, ParseError> {
    let prefix_expr = PrefixExpression::parse(lexer, alloc)?;
    match prefix_expr {
        PrefixExpression::FnCall(call) => Ok(Statement::Call(alloc.alloc(call))),
        PrefixExpression::Variable(head) => {
            let assign = Assignment::parse(head, lexer, alloc)?;
            Ok(Statement::Assignment(alloc.alloc(assign)))
        }
        PrefixExpression::Parenthesized(_) => Err(ParseError::unrecoverable_from_here(
            lexer,
            SyntaxError::ExpectedVarOrCall,
        )),
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
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(stat, Statement::Empty(Empty));

        Ok(())
    }

    #[test]
    fn parses_break_stat() -> anyhow::Result<()> {
        let src = "break";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(stat, Statement::Break(Break));

        Ok(())
    }

    #[test]
    fn parses_label_stat() -> anyhow::Result<()> {
        let src = "::foo::";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(stat, Statement::Label(Label(Ident(0))));

        Ok(())
    }

    #[test]
    fn parses_goto_stat() -> anyhow::Result<()> {
        let src = "goto foo";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(stat, Statement::Goto(Goto(Ident(0))));

        Ok(())
    }

    #[test]
    fn parses_goto_handles_name() -> anyhow::Result<()> {
        let src = "goto gotofoo";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(stat, Statement::Goto(Goto(Ident(0))));

        Ok(())
    }

    #[test]
    fn parses_do_stat() -> anyhow::Result<()> {
        let src = "do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let stat = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

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
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(
            result,
            Statement::Assignment(&Assignment {
                varlist: List::new(&mut ListNode::new(VarPrefixExpression::Name(Ident(0)))),
                expressions: List::new(&mut ListNode::new(Expression::Number(Number::Integer(10)))),
            })
        );

        Ok(())
    }

    #[test]
    fn parses_multi() -> anyhow::Result<()> {
        let src = "a ,b, c , d = 10, 11, 12 , 13";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => Statement::parse)?;

        assert_eq!(
            result,
            Statement::Assignment(&Assignment {
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
            })
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
