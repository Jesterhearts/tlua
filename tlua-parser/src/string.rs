use atoi::FromRadix16Checked;
use derive_more::Deref;
use internment::LocalIntern;
use nom::{
    branch::alt,
    bytes::complete::{
        is_a,
        is_not,
        tag,
        take_till,
        take_while,
    },
    character::complete::{
        char as token,
        hex_digit1,
        satisfy,
    },
    combinator::{
        map,
        map_res,
        opt,
        recognize,
        value,
    },
    multi::many1_count,
    sequence::{
        delimited,
        pair,
        preceded,
        terminated,
        tuple,
    },
    Slice,
};

use crate::{
    identifiers::Ident,
    lua_whitespace0,
    ParseResult,
    Span,
    SyntaxError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct ConstantString(pub(crate) LocalIntern<Vec<u8>>);

impl From<Ident> for ConstantString {
    fn from(ident: Ident) -> Self {
        Self(ident.0)
    }
}

impl From<&'_ Ident> for ConstantString {
    fn from(ident: &Ident) -> Self {
        Self(ident.0)
    }
}

impl ConstantString {
    pub fn new(data: Vec<u8>) -> Self {
        Self(LocalIntern::new(data))
    }

    pub fn data(&self) -> &Vec<u8> {
        &*self.0
    }
}

impl From<&str> for ConstantString {
    fn from(s: &str) -> Self {
        Self::new(Vec::from(s.as_bytes()))
    }
}

impl PartialEq<&str> for ConstantString {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_slice() == other.as_bytes()
    }
}

/// Parses a lua string starting at offset 0 of the input span according to
/// https://www.lua.org/manual/5.4/manual.html#3.1
/// A string without escapes would match (PCRE2):
/// - regex `'[^\n']*'`
/// - regex `\"[^\n"]*\"`
/// - regex `\[(=*)\[.*?\]\1\]`
/// See the internal documentation for the exact details of string parsing.
pub fn parse_string(input: Span) -> ParseResult<ConstantString> {
    alt((
        preceded(token('\''), parse_remaining_quoted_string::<b'\''>),
        preceded(token('"'), parse_remaining_quoted_string::<b'"'>),
        parse_raw_string,
    ))(input)
}

/// Expects an input string with the opening delimiter already skipped.
/// Parses the string by:
///   - Consume characters until a `\`, `<ascii newline 0xA>`, or DELIMITER is
///     encountered.
///   - If the encountered character is DELIMITER, we know that we would have
///     already encountered & handled any escaping, so our string is complete.
///   - If the character is not the DELIMITER, it is either a literal `\` and we
///     should parse an escape sequence or it is a newline - which is an error
///     and will be handled when we fail to match an escape sequence.
///   - Match any of the escape sequences listed in the LUA spec and add those
///     bytes to the string or return an error.
///   - goto 1.
fn parse_remaining_quoted_string<const DELIMITER: u8>(input: Span) -> ParseResult<ConstantString> {
    let mut output = Vec::default();

    let mut remain = input;
    loop {
        let (pending, consumed) =
            take_till(|c: u8| c == DELIMITER || matches!(c, b'\\' | b'\n'))(remain)?;
        remain = pending;

        output.extend_from_slice(*consumed);

        if remain.starts_with(&[DELIMITER]) {
            return Ok((remain.slice(1..), ConstantString::new(output)));
        }

        #[derive(Clone, Copy)]
        enum EscapedValue {
            Byte(u8),
            Unicode { len: usize, bytes: [u8; 6] },
        }

        let (pending, maybe_escaped) = alt((
            value(Some(EscapedValue::Byte(b'\x07')), tag(r#"\a"#)),
            value(Some(EscapedValue::Byte(b'\x08')), tag(r#"\b"#)),
            value(Some(EscapedValue::Byte(b'\x0C')), tag(r#"\f"#)),
            value(Some(EscapedValue::Byte(b'\n')), tag(r#"\n"#)),
            value(Some(EscapedValue::Byte(b'\r')), tag(r#"\r"#)),
            value(Some(EscapedValue::Byte(b'\t')), tag(r#"\t"#)),
            value(Some(EscapedValue::Byte(b'\x0B')), tag(r#"\v"#)),
            value(Some(EscapedValue::Byte(b'\\')), tag(r#"\\"#)),
            value(Some(EscapedValue::Byte(b'\"')), tag(r#"\""#)),
            value(Some(EscapedValue::Byte(b'\'')), tag(r#"\'"#)),
            value(
                Some(EscapedValue::Byte(b'\n')),
                alt((tag("\\\r\n"), tag("\\\n\r"), tag("\\\n"), tag("\\\r"))),
            ),
            value(None, terminated(tag(r#"\z"#), lua_whitespace0)),
            map(
                preceded(
                    tag(r#"\x"#),
                    pair(
                        satisfy(|c| c.is_ascii_hexdigit()),
                        satisfy(|c| c.is_ascii_hexdigit()),
                    ),
                ),
                |(hex1, hex2)| {
                    let high = hex1.to_digit(16).expect("Tested ascii hexdigit");
                    let low = hex2.to_digit(16).expect("Tested ascii hexdigit");

                    Some(EscapedValue::Byte((high * 16 + low) as u8))
                },
            ),
            map_res::<_, _, _, _, SyntaxError, _, _>(
                delimited(tag(r#"\u{"#), hex_digit1, token('}')),
                |seq: Span| {
                    let (len, bytes) = encode_utf8_raw(seq)?;

                    Ok(Some(EscapedValue::Unicode { len, bytes }))
                },
            ),
            map_res(
                preceded(
                    token('\\'),
                    tuple((
                        satisfy(|c| c.is_ascii_digit()),
                        satisfy(|c| c.is_ascii_digit()),
                        satisfy(|c| c.is_ascii_digit()),
                    )),
                ),
                |(d1, d2, d3)| {
                    let hundreds = d1.to_digit(10).expect("Tested ascii digit");
                    let tens = d2.to_digit(10).expect("Tested ascii digit");
                    let ones = d3.to_digit(10).expect("Tested ascii digit");

                    let result = hundreds * 100 + tens * 10 + ones;
                    if result > u8::MAX.into() {
                        return Err(SyntaxError::DecimalEscapeTooLarge);
                    }

                    Ok(Some(EscapedValue::Byte(result as u8)))
                },
            ),
        ))(remain)?;
        remain = pending;

        match maybe_escaped {
            Some(EscapedValue::Byte(b)) => output.push(b),
            Some(EscapedValue::Unicode { len, bytes }) => output.extend_from_slice(&bytes[..len]),
            None => (),
        }
    }
}

/// Encodes a a 4-byte sequence of hex characters into a (potentially invalid -
/// per spec) utf8 byte sequence.
fn encode_utf8_raw(seq: Span) -> Result<(usize, [u8; 6]), SyntaxError> {
    let (val, _) = u32::from_radix_16_checked(*seq);
    let val = if let Some(val) = val {
        val
    } else {
        return Err(SyntaxError::Utf8ValueTooLarge);
    };

    #[rustfmt::skip]
    mod tag {
    pub const CONT: u8    = 0b10000000;
    pub const TWO_B: u8   = 0b11000000;
    pub const THREE_B: u8 = 0b11100000;
    pub const FOUR_B: u8  = 0b11110000;
    pub const FIVE_B: u8  = 0b11111000;
    pub const SIX_B: u8   = 0b11111100;
    }

    #[rustfmt::skip]
    mod mask {
    pub const CONT: u32    = 0b00111111;
    pub const TWO_B: u32   = 0b00011111;
    pub const THREE_B: u32 = 0b00001111;
    pub const FOUR_B: u32  = 0b00000111;
    pub const FIVE_B: u32  = 0b00000011;
    pub const SIX_B: u32   = 0b00000001;
    }

    // These groupings are based on the layout of utf8 encoding, not bytes.
    #[allow(clippy::unusual_byte_groupings)]
    #[rustfmt::skip]
    mod max {
    pub const ONE_B: u32   = 0b10000000;
    pub const TWO_B: u32   = 0b00100000__000000;
    pub const THREE_B: u32 = 0b00010000__000000__000000;
    pub const FOUR_B: u32  = 0b00001000__000000__000000__000000;
    pub const FIVE_B: u32  = 0b00000100__000000__000000__000000__000000;
    pub const SIX_B: u32   = 0b00000010__000000__000000__000000__000000__000000;
    }

    let len = if val < max::ONE_B {
        1
    } else if val < max::TWO_B {
        2
    } else if val < max::THREE_B {
        3
    } else if val < max::FOUR_B {
        4
    } else if val < max::FIVE_B {
        5
    } else if val < max::SIX_B {
        6
    } else {
        return Err(SyntaxError::Utf8ValueTooLarge);
    };

    let bytes = match len {
        1 => [val as u8, 0, 0, 0, 0, 0],
        2 => [
            (val >> 6 & mask::TWO_B) as u8 | tag::TWO_B,
            (val & mask::CONT) as u8 | tag::CONT,
            0,
            0,
            0,
            0,
        ],
        3 => [
            (val >> 12 & mask::THREE_B) as u8 | tag::THREE_B,
            (val >> 6 & mask::CONT) as u8 | tag::CONT,
            (val & mask::CONT) as u8 | tag::CONT,
            0,
            0,
            0,
        ],
        4 => [
            (val >> 18 & mask::FOUR_B) as u8 | tag::FOUR_B,
            (val >> 12 & mask::CONT) as u8 | tag::CONT,
            (val >> 6 & mask::CONT) as u8 | tag::CONT,
            (val & mask::CONT) as u8 | tag::CONT,
            0,
            0,
        ],
        5 => [
            (val >> 24 & mask::FIVE_B) as u8 | tag::FIVE_B,
            (val >> 18 & mask::CONT) as u8 | tag::CONT,
            (val >> 12 & mask::CONT) as u8 | tag::CONT,
            (val >> 6 & mask::CONT) as u8 | tag::CONT,
            (val & mask::CONT) as u8 | tag::CONT,
            0,
        ],
        6 => [
            (val >> 30 & mask::SIX_B) as u8 | tag::SIX_B,
            (val >> 24 & mask::CONT) as u8 | tag::CONT,
            (val >> 18 & mask::CONT) as u8 | tag::CONT,
            (val >> 12 & mask::CONT) as u8 | tag::CONT,
            (val >> 6 & mask::CONT) as u8 | tag::CONT,
            (val & mask::CONT) as u8 | tag::CONT,
        ],
        _ => unreachable!(),
    };

    Ok((len, bytes))
}

/// Loops over the input until it encounters a `]` followed by sep followed by a
/// `]`. This is used in concert with nom::recognize to extract the raw string.
fn discard_raw_str<'a>(mut pending: Span<'a>, sep: &'a [u8]) -> ParseResult<'a, ()> {
    loop {
        pending = take_till(|c| c == b']')(pending)?.0;

        if pending[1..].starts_with(sep) && pending[1 + sep.len()..].starts_with(&[b']']) {
            return Ok((pending, ()));
        }

        pending = pair(tag(b"]"), opt(is_a(sep)))(pending)?.0;
    }
}

fn parse_raw_string(input: Span) -> ParseResult<ConstantString> {
    let (remain, open) = delimited(token('['), take_while(|c| c == b'='), token('['))(input)?;
    let (remain, data) = recognize(|input| discard_raw_str(input, *open))(remain)?;
    let (remain, ()) = value((), delimited(token(']'), tag(*open), token(']')))(remain)?;

    let mut result = Vec::default();
    let mut data = opt(alt((tag("\r\n"), tag("\n\r"), tag("\n"), tag("\r"))))(data)?.0;

    loop {
        if data.is_empty() {
            return Ok((remain, ConstantString::new(result)));
        }

        let (pending, part) = is_not("\r\n")(data)?;
        data = pending;

        result.extend_from_slice(*part);

        if data.is_empty() {
            return Ok((remain, ConstantString::new(result)));
        }

        let (pending, newlines) =
            many1_count(alt((tag("\r\n"), tag("\n\r"), tag("\n"), tag("\r"))))(data)?;
        data = pending;
        result.extend(std::iter::repeat(b'\n').take(newlines));
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::parse_string;
    use crate::{
        final_parser,
        string::ConstantString,
        Span,
    };

    #[test]
    pub fn single_quoted() -> anyhow::Result<()> {
        let src = r#"'a string'"#;

        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "a string");

        Ok(())
    }

    #[test]
    pub fn double_quoted() -> anyhow::Result<()> {
        let src = r#""a string""#;

        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "a string");

        Ok(())
    }

    #[test]
    pub fn escape_bell() -> anyhow::Result<()> {
        let src = r#""\a""#;

        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\x07");

        Ok(())
    }

    #[test]
    pub fn escape_backspace() -> anyhow::Result<()> {
        let src = r#""\b""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\x08");

        Ok(())
    }

    #[test]
    pub fn escape_formfeed() -> anyhow::Result<()> {
        let src = r#""\f""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\x0C");

        Ok(())
    }

    #[test]
    pub fn escape_newline() -> anyhow::Result<()> {
        let src = r#""\n""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\n");

        Ok(())
    }

    #[test]
    pub fn escape_carriage_return() -> anyhow::Result<()> {
        let src = r#""\r""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\r");

        Ok(())
    }

    #[test]
    pub fn escape_tab() -> anyhow::Result<()> {
        let src = r#""\t""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\t");

        Ok(())
    }

    #[test]
    pub fn escape_vertical_tab() -> anyhow::Result<()> {
        let src = r#""\v""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\x0B");

        Ok(())
    }

    #[test]
    pub fn escape_backslash() -> anyhow::Result<()> {
        let src = r#""\\""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\\");

        Ok(())
    }

    #[test]
    pub fn escape_double_quote() -> anyhow::Result<()> {
        let src = r#""\"""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\"");

        Ok(())
    }

    #[test]
    pub fn escape_single_quote() -> anyhow::Result<()> {
        let src = r#""\'""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\'");

        Ok(())
    }

    #[test]
    pub fn escape_z() -> anyhow::Result<()> {
        let src = r#""\z  
            a string""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "a string");

        Ok(())
    }

    #[test]
    pub fn escape_z_empty() -> anyhow::Result<()> {
        let src = r#""\za string""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "a string");

        Ok(())
    }

    #[test]
    pub fn escape_continuation_newline() -> anyhow::Result<()> {
        let src = "\"line1 \\\n a string\"";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "line1 \n a string");

        Ok(())
    }

    #[test]
    pub fn escape_continuation_carriage_return() -> anyhow::Result<()> {
        let src = "\"line1 \\\r a string\"";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "line1 \n a string");

        Ok(())
    }

    #[test]
    pub fn escape_continuation_carriage_return_newline() -> anyhow::Result<()> {
        let src = "\"line1 \\\r\n a string\"";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "line1 \n a string");

        Ok(())
    }

    #[test]
    pub fn escape_continuation_newline_carriage_return() -> anyhow::Result<()> {
        let src = "\"line1 \\\n\r a string\"";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "line1 \n a string");

        Ok(())
    }

    #[test]
    pub fn escape_unicode() -> anyhow::Result<()> {
        let src = r#""\u{2764}""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "\u{2764}");

        Ok(())
    }

    #[test]
    pub fn escape_unicode_allow_invalid() -> anyhow::Result<()> {
        let src = r#""\u{7FFFFFFF}""#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(
            result,
            ConstantString::new(vec![253, 191, 191, 191, 191, 191])
        );

        Ok(())
    }

    #[test]
    pub fn raw_string() -> anyhow::Result<()> {
        let src = r#"[[]]"#;
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    pub fn raw_string_extended() -> anyhow::Result<()> {
        let src = "[===[]===]";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    pub fn raw_string_extended_unbalanced() -> anyhow::Result<()> {
        let src = "[===[]==]===]";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "]==");

        Ok(())
    }

    #[test]
    pub fn raw_string_first_newline_skip() -> anyhow::Result<()> {
        let src = "[===[\n]===]";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(result, "");

        Ok(())
    }

    #[test]
    pub fn raw_string_newline_replace() -> anyhow::Result<()> {
        let src = "[===[\nline1\nline2\r\nline3\n\rline4\rline5\n\nline6\r\rline7\r\n\nline8\r\n\rline9\nline10\n\r\r\n\n\rline11]===]";
        let result = final_parser!(Span::new(src.as_bytes()) => parse_string)?;

        assert_eq!(
            result,
            "line1\nline2\nline3\nline4\nline5\n\nline6\n\nline7\n\nline8\n\nline9\nline10\n\n\nline11"
        );

        Ok(())
    }
}
