use std::ops::Deref;

use internment::LocalIntern;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1,
        alphanumeric0,
    },
    combinator::{
        map_res,
        opt,
        recognize,
    },
    sequence::{
        delimited,
        pair,
        preceded,
    },
};
use nom_supreme::ParserExt;

use crate::{
    is_keyword,
    list::List,
    lua_whitespace0,
    ASTAllocator,
    ParseResult,
    Span,
    SyntaxError,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(pub(crate) LocalIntern<Vec<u8>>);

impl Deref for Ident {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl Ident {
    pub fn new_from_slice(data: &[u8]) -> Self {
        let mut vec = Vec::with_capacity(data.len());
        vec.extend_from_slice(data);
        Self(LocalIntern::new(vec))
    }

    pub(crate) fn parser(
        _: &'_ ASTAllocator,
    ) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, Ident> + '_ {
        |input| {
            map_res(
                recognize(pair(alt((tag("_"), alpha1)), alphanumeric0)).context("identifier"),
                |raw_ident| {
                    if is_keyword(raw_ident) {
                        Err(SyntaxError::KeywordAsIdent)
                    } else {
                        Ok(Ident::new_from_slice(*raw_ident))
                    }
                },
            )(input)
        }
    }
}

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ident")
            .field(&String::from_utf8_lossy(&*self.0))
            .finish()
    }
}

impl<'chunk> ToString for Ident {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&*self.0).to_string()
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Self::new_from_slice(s.as_bytes())
    }
}

pub fn identifier_list1<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, Ident>> {
    let (remain, head) = Ident::parser(alloc)(input)?;
    input = remain;

    let mut list = List::default();
    let mut current = list.cursor_mut().alloc_insert_advance(alloc, head);

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, tag(","), lua_whitespace0),
            Ident::parser(alloc),
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
        final_parser,
        identifiers::Ident,
        ASTAllocator,
        Span,
    };

    #[test]
    pub fn parses_ident() -> anyhow::Result<()> {
        let ident = "_";

        let alloc = ASTAllocator::default();
        let ident = final_parser!(Span::new(ident.as_bytes()) => Ident::parser(&alloc))?;

        assert_eq!(ident, "_".into());

        Ok(())
    }

    #[test]
    pub fn parses_ident_alpha_start() -> anyhow::Result<()> {
        let ident = "a";

        let alloc = ASTAllocator::default();
        let ident = final_parser!(Span::new(ident.as_bytes()) => Ident::parser(&alloc))?;

        assert_eq!(ident, "a".into());

        Ok(())
    }

    #[test]
    pub fn parses_ident_rejects_num_start() {
        let ident = "9";

        let alloc = ASTAllocator::default();
        assert!(final_parser!(Span::new(ident.as_bytes()) => Ident::parser(&alloc)).is_err());
    }

    #[test]
    pub fn parses_ident_rejects_keyword() {
        let ident = "while";

        let alloc = ASTAllocator::default();
        assert!(final_parser!(Span::new(ident.as_bytes()) => Ident::parser(&alloc)).is_err());
    }

    #[test]
    pub fn parses_ident_alphanum() -> anyhow::Result<()> {
        let ident = "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let alloc = ASTAllocator::default();
        let ident = final_parser!(Span::new(ident.as_bytes()) => Ident::parser(&alloc))?;

        assert_eq!(
            ident,
            "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".into()
        );

        Ok(())
    }
}
