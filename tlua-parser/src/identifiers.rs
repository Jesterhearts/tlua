use std::ops::Deref;

use internment::LocalIntern;
use nom::{
    branch::alt,
    character::complete::{
        alpha1,
        char as token,
        one_of,
    },
    combinator::{
        map_res,
        not,
        recognize,
        value,
    },
    multi::many0_count,
    sequence::{
        delimited,
        pair,
        terminated,
    },
};
use nom_supreme::tag::complete::tag;

use crate::{
    build_separated_list1,
    expecting,
    is_keyword,
    list::List,
    lua_whitespace0,
    ASTAllocator,
    ParseResult,
    Span,
    SyntaxError,
};

const WORD_CHARS: &str = "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

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
                expecting(
                    recognize(pair(
                        alt((tag("_"), alpha1)),
                        many0_count(one_of(WORD_CHARS)),
                    )),
                    "ident",
                ),
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

pub fn keyword(kw: &'static str) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, ()> {
    move |input| expecting(value((), terminated(tag(kw), not(one_of(WORD_CHARS)))), kw)(input)
}

pub fn build_identifier_list1<'chunk>(
    alloc: &'chunk ASTAllocator,
) -> impl for<'src> FnMut(Span<'src>) -> ParseResult<'src, List<'chunk, Ident>> {
    |input| {
        build_separated_list1(
            alloc,
            Ident::parser(alloc),
            delimited(lua_whitespace0, token(','), lua_whitespace0),
        )(input)
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
        let ident = "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_";

        let alloc = ASTAllocator::default();
        let ident = final_parser!(Span::new(ident.as_bytes()) => Ident::parser(&alloc))?;

        assert_eq!(
            ident,
            "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_".into()
        );

        Ok(())
    }
}
