use crate::{
    block::Block,
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
    ParseErrorExt,
    PeekableLexer,
};

#[derive(Debug, PartialEq)]
pub struct ForEachLoop<'chunk> {
    pub vars: List<'chunk, Ident>,
    pub expressions: List<'chunk, Expression<'chunk>>,
    pub body: Block<'chunk>,
}

impl<'chunk> ForEachLoop<'chunk> {
    pub(crate) fn parse(
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let for_kw = lexer.expecting_token(Token::KWfor)?;

        let vars = match Ident::parse_list1(lexer, alloc) {
            Ok(idents) => idents,
            Err(e) => {
                lexer.reset(for_kw);
                return Err(e);
            }
        };

        lexer
            .expecting_token(Token::KWin)
            .reset_on_err(lexer, for_kw)?;

        let expressions = Expression::parse_list1(lexer, alloc).mark_unrecoverable()?;

        let body = Block::parse_do(lexer, alloc).mark_unrecoverable()?;

        Ok(Self {
            vars,
            expressions,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::ForEachLoop;
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
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_foreach() -> anyhow::Result<()> {
        let src = "for a,b,c,d in 1,2,3,4 do end";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = final_parser!((src.as_bytes(), &alloc, &mut strings) => ForEachLoop::parse)?;

        assert_eq!(
            result,
            ForEachLoop {
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
            }
        );

        Ok(())
    }
}
