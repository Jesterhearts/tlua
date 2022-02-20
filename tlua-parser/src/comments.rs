use nom::{
    branch::alt,
    bytes::complete::{
        tag,
        take_till,
        take_until,
    },
    combinator::{
        success,
        value,
    },
    sequence::{
        delimited,
        preceded,
    },
};
use nom_supreme::ParserExt;

use crate::{
    ParseResult,
    Span,
};

pub fn parse_comment(input: Span) -> ParseResult<()> {
    alt((
        value(
            (),
            delimited(
                tag("--[["),
                alt((value((), take_until("]]")), success(()))),
                tag("]]"),
            ),
        )
        .context("multiline comment"),
        value(
            (),
            preceded(tag("--"), take_till(|c| c == b'\r' || c == b'\n')),
        )
        .context("comment"),
    ))(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::parse_comment;
    use crate::Span;

    #[test]
    pub fn parses_empty_short_comment() -> anyhow::Result<()> {
        let comment = "--";

        let (remain, ()) = parse_comment(Span::new(comment.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");

        Ok(())
    }

    #[test]
    pub fn parses_short_comment() -> anyhow::Result<()> {
        let comment = "--abcdef";

        let (remain, ()) = parse_comment(Span::new(comment.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");

        Ok(())
    }

    #[test]
    pub fn parses_empty_long_comment() -> anyhow::Result<()> {
        let comment = "--[[]]";

        let (remain, ()) = parse_comment(Span::new(comment.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");

        Ok(())
    }

    #[test]
    pub fn parses_long_comment() -> anyhow::Result<()> {
        let comment = "--[[abc
            def
        ghi]]";

        let (remain, ()) = parse_comment(Span::new(comment.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");

        Ok(())
    }
}
