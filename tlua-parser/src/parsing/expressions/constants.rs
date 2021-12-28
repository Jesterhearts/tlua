use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::value,
};

use crate::{
    ast::expressions::Nil,
    parsing::{
        ParseResult,
        Span,
    },
};

pub fn parse_nil(input: Span) -> ParseResult<Nil> {
    value(Nil, tag("nil"))(input)
}

pub fn parse_bool(input: Span) -> ParseResult<bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        ast::expressions::Nil,
        parsing::{
            expressions::constants::{
                parse_bool,
                parse_nil,
            },
            Span,
        },
    };

    #[test]
    pub fn parses_nil() -> anyhow::Result<()> {
        let nil = "nil";

        let (remain, nil) = parse_nil(Span::new(nil.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(nil, Nil);

        Ok(())
    }

    #[test]
    pub fn parses_true() -> anyhow::Result<()> {
        let lit_true = "true";

        let (remain, lit_true) = parse_bool(Span::new(lit_true.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert!(lit_true);

        Ok(())
    }

    #[test]
    pub fn parses_false() -> anyhow::Result<()> {
        let lit_false = "false";

        let (remain, lit_false) = parse_bool(Span::new(lit_false.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert!(!lit_false);

        Ok(())
    }
}
