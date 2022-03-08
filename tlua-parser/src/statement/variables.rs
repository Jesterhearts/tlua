use crate::{
    combinators::parse_separated_list_with_head,
    expressions::Expression,
    identifiers::Ident,
    lexer::Token,
    list::List,
    ASTAllocator,
    ParseError,
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
    ) -> Result<Option<Self>, ParseError> {
        Ident::parse(lexer, alloc).map_or(Ok(None), |name| {
            Self::parse_remaining(name, lexer, alloc).map(Some)
        })
    }

    pub(crate) fn parse_remaining(
        name: Ident,
        lexer: &mut PeekableLexer,
        _: &ASTAllocator,
    ) -> Result<Self, ParseError> {
        let attribute = if lexer.next_if_eq(Token::LeftAngle).is_some() {
            let ident = lexer.expecting_token(Token::Ident)?;
            let attrib = match ident.src {
                b"const" => Attribute::Const,
                b"close" => Attribute::Close,
                _ => {
                    return Err(ParseError::from_here(lexer, SyntaxError::InvalidAttribute));
                }
            };
            lexer.expecting_token(Token::RightAngle)?;
            Some(attrib)
        } else {
            None
        };

        Ok(Self { name, attribute })
    }
}

impl<'chunk> LocalVarList<'chunk> {
    pub(crate) fn parse_remaining(
        head_ident: Ident,
        lexer: &mut PeekableLexer,
        alloc: &'chunk ASTAllocator,
    ) -> Result<Self, ParseError> {
        let head = LocalVar::parse_remaining(head_ident, lexer, alloc)?;
        let vars = parse_separated_list_with_head(head, lexer, alloc, LocalVar::parse, |token| {
            *token == Token::Comma
        })?;

        let initializers = if lexer.next_if_eq(Token::Equals).is_some() {
            Expression::parse_list1(lexer, alloc)?
        } else {
            Default::default()
        };

        Ok(Self { vars, initializers })
    }
}
