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
use tracing::instrument;

use crate::{
    ast::identifiers::Ident,
    list::List,
    parsing::{
        is_keyword,
        lua_whitespace0,
        ASTAllocator,
        ParseResult,
        Span,
        SyntaxError,
    },
};

pub(crate) fn parse_identifier<'src, 'chunk>(
    input: Span<'src>,
    _alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, Ident> {
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

#[instrument(level = "trace", name = "ident_list", skip(input, alloc))]
pub(crate) fn identifier_list1<'src, 'chunk>(
    mut input: Span<'src>,
    alloc: &'chunk ASTAllocator,
) -> ParseResult<'src, List<'chunk, Ident>> {
    let (remain, head) = parse_identifier(input, alloc)?;
    input = remain;

    let mut list = List::default();
    let mut current = list.cursor_mut().alloc_insert_advance(alloc, head);

    loop {
        let (remain, maybe_next) = opt(preceded(
            delimited(lua_whitespace0, tag(","), lua_whitespace0),
            |input| parse_identifier(input, alloc),
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

    use super::parse_identifier;
    use crate::parsing::{
        ASTAllocator,
        Span,
    };

    #[test]
    pub(crate) fn parses_ident() -> anyhow::Result<()> {
        let ident = "_";

        let alloc = ASTAllocator::default();
        let (remain, ident) = parse_identifier(Span::new(ident.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(ident, "_".into());

        Ok(())
    }

    #[test]
    pub(crate) fn parses_ident_alpha_start() -> anyhow::Result<()> {
        let ident = "a";

        let alloc = ASTAllocator::default();
        let (remain, ident) = parse_identifier(Span::new(ident.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(ident, "a".into());

        Ok(())
    }

    #[test]
    pub(crate) fn parses_ident_rejects_num_start() {
        let ident = "9";

        let alloc = ASTAllocator::default();
        assert!(parse_identifier(Span::new(ident.as_bytes()), &alloc).is_err());
    }

    #[test]
    pub(crate) fn parses_ident_rejects_keyword() {
        let ident = "while";

        let alloc = ASTAllocator::default();
        assert!(parse_identifier(Span::new(ident.as_bytes()), &alloc).is_err());
    }

    #[test]
    pub(crate) fn parses_ident_alphanum() -> anyhow::Result<()> {
        let ident = "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let alloc = ASTAllocator::default();
        let (remain, ident) = parse_identifier(Span::new(ident.as_bytes()), &alloc)?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(
            ident,
            "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".into()
        );

        Ok(())
    }
}
