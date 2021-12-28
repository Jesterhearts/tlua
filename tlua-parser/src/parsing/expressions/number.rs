use atoi::atoi;
use hexf_parse::parse_hexf64;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        char as token,
        digit0,
        digit1,
        hex_digit0,
        hex_digit1,
        one_of,
    },
    combinator::{
        map,
        map_res,
        opt,
        recognize,
        value,
    },
    sequence::{
        pair,
        preceded,
        tuple,
    },
};

use crate::{
    ast::expressions::number::Number,
    parsing::{
        ParseResult,
        Span,
        SyntaxError,
    },
};

/// Locale-insensitive parsing for numbers.
pub fn parse_number(input: Span) -> ParseResult<Number> {
    alt((
        map(parse_hex_float, Number::Float),
        map(parse_float, Number::Float),
        map(parse_hex_integer, Number::Integer),
        parse_integer,
    ))(input)
}

pub fn parse_float(input: Span) -> ParseResult<f64> {
    map_res(
        recognize(tuple((
            opt(token('-')),
            digit0,
            // We try to match either a mandatory decimal part + an optional exponent part or a
            // mandatory exponent part here in order to disambiguate with integer parsing.
            // If tried to match using an optional for both (which is more natural), we would
            // recognize e.g. 10 as a floating point number instead of an integer.
            alt((
                value(
                    (),
                    pair(
                        pair(token('.'), digit0),
                        opt(tuple((one_of("eE"), opt(token('-')), digit1))),
                    ),
                ),
                value((), tuple((one_of("eE"), opt(token('-')), digit1))),
            )),
        ))),
        |float_str: Span| {
            // SAFETY: We know that we only have [-]<decimal digits>[.[<decimal
            // digits>]][<eE>[-]<decimal digits>] in our float string.
            unsafe { std::str::from_utf8_unchecked(*float_str) }
                .parse::<f64>()
                .map_err(|_| SyntaxError::MalformedNumber)
        },
    )(input)
}

pub fn parse_hex_float(input: Span) -> ParseResult<f64> {
    map_res(
        recognize(tuple((
            opt(token('-')),
            tag("0x"),
            hex_digit0,
            // The same reasoning for this odd alt matching applies here as for regular floating
            // point parsing. We need to disambiguate with hexidecimal integer constant parsing.
            alt((
                value(
                    (),
                    pair(
                        pair(token('.'), hex_digit0),
                        opt(tuple((one_of("pP"), opt(token('-')), hex_digit1))),
                    ),
                ),
                value((), tuple((one_of("pP"), opt(token('-')), hex_digit1))),
            )),
        ))),
        |float_str: Span| {
            // SAFETY: We know that we only have [-]0x<hex digits>[.[<hex digits>]][p[-]<hex
            // digits>] in our float string.
            parse_hexf64(unsafe { std::str::from_utf8_unchecked(*float_str) }, false)
                .map_err(|_| SyntaxError::MalformedNumber)
        },
    )(input)
}

/// Parses an integer, falling back to a float if the integer would overflow.
pub fn parse_integer<'src>(input: Span<'src>) -> ParseResult<'src, Number> {
    alt((
        map_res(
            recognize(pair(opt(token('-')), digit1)),
            |num_str: Span<'src>| {
                if let Some(i) = atoi::<i64>(*num_str).map(Number::Integer) {
                    Ok(i)
                } else {
                    Err(SyntaxError::IntegerConstantTooLarge)
                }
            },
        ),
        map(parse_float, Number::Float),
    ))(input)
}

pub fn parse_hex_integer(input: Span) -> ParseResult<i64> {
    let (input, digits) = preceded(tag("0x"), hex_digit1)(input)?;

    let mut result: i64 = 0;
    for digit in digits.iter().copied().map(|d| match d {
        b'0' => 0,
        b'1' => 1,
        b'2' => 2,
        b'3' => 3,
        b'4' => 4,
        b'5' => 5,
        b'6' => 6,
        b'7' => 7,
        b'8' => 8,
        b'9' => 9,
        b'a' | b'A' => 10,
        b'b' | b'B' => 11,
        b'c' | b'C' => 12,
        b'd' | b'D' => 13,
        b'e' | b'E' => 14,
        b'f' | b'F' => 15,
        _ => unreachable!(),
    }) {
        result = result.wrapping_mul(16).wrapping_add(digit)
    }

    Ok((input, result))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{
        parse_hex_integer,
        parse_number,
    };
    use crate::{
        ast::expressions::number::Number,
        parsing::Span,
    };

    #[test]
    pub fn parses_float_constant() -> anyhow::Result<()> {
        let float = "1.";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(1.0));

        Ok(())
    }

    #[test]
    pub fn parses_float_constant_decimals() -> anyhow::Result<()> {
        let float = "1.00";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(1.0));

        Ok(())
    }

    #[test]
    pub fn parses_float_constant_exponent() -> anyhow::Result<()> {
        let float = "1.e10";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(1.0e10));

        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    pub fn parses_float_constant_Exponent() -> anyhow::Result<()> {
        let float = "1.E10";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(1.0e10));

        Ok(())
    }

    #[test]
    pub fn parses_float_constant_neg_exponent() -> anyhow::Result<()> {
        let float = "1.e-10";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(1.0e-10));

        Ok(())
    }

    #[test]
    #[allow(non_snake_case)]
    pub fn parses_float_constant_neg_Exponent() -> anyhow::Result<()> {
        let float = "1.E-10";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(1.0e-10));

        Ok(())
    }

    #[test]
    pub fn parses_hex_float_constant() -> anyhow::Result<()> {
        let float = "0x1F.0p-1";

        let (remain, number) = parse_number(Span::new(float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Float(15.5));

        Ok(())
    }

    #[test]
    pub fn parses_integer_constant() -> anyhow::Result<()> {
        let will_round_if_parsed_as_float = "9007199254740993";

        let (remain, number) = parse_number(Span::new(will_round_if_parsed_as_float.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Integer(9007199254740993));

        Ok(())
    }

    #[test]
    pub fn parses_negative_integer_constant() -> anyhow::Result<()> {
        let negative = "-10";

        let (remain, number) = parse_number(Span::new(negative.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, Number::Integer(-10));

        Ok(())
    }

    #[test]
    pub fn parses_hex_constant_wrapping() -> anyhow::Result<()> {
        let big_const = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFFFFFFFFFFF";

        let (remain, number) = parse_hex_integer(Span::new(big_const.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, -1152921504606846977);

        Ok(())
    }

    #[test]
    pub fn parses_hex_constant_wrapping_2() -> anyhow::Result<()> {
        let big_const = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

        let (remain, number) = parse_hex_integer(Span::new(big_const.as_bytes()))?;

        assert_eq!(std::str::from_utf8(*remain)?, "");
        assert_eq!(number, -1);

        Ok(())
    }
}
