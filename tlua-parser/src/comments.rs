use nom::{
    branch::alt,
    bytes::complete::{
        tag,
        take_till,
        take_until,
    },
    combinator::{
        cut,
        success,
        value,
    },
    sequence::{
        pair,
        preceded,
    },
};

use crate::{
    ParseResult,
    Span,
};

pub fn parse_comment(input: Span) -> ParseResult<()> {
    alt((
        value(
            (),
            preceded(
                tag("--[["),
                cut(pair(
                    alt((value((), take_until("]]")), success(()))),
                    tag("]]"),
                )),
            ),
        ),
        value(
            (),
            preceded(tag("--"), take_till(|c| c == b'\r' || c == b'\n')),
        ),
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
