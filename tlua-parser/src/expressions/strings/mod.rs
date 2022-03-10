mod multiline_strings;

use atoi::FromRadix16Checked;
use bstr::BString;
use logos::{
    Lexer,
    Logos,
};

use crate::{
    identifiers::Ident,
    lexer::Token,
    token_subset,
    ASTAllocator,
    ParseError,
    PeekableLexer,
    SyntaxError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantString(pub(crate) usize);

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

token_subset! {
    StringStart {
        Token::SingleQuotedStringStart,
        Token::DoubleQuotedStringStart,
        Error(SyntaxError::ExpectedString)
    }
}

impl ConstantString {
    pub(crate) fn try_parse(
        lexer: &mut PeekableLexer,
        _: &ASTAllocator,
    ) -> Result<Option<ConstantString>, ParseError> {
        let token = if let Some(token) = StringStart::next(lexer) {
            token
        } else {
            return multiline_strings::parse_string(lexer);
        };

        let remain = lexer.remainder();
        let mut string_lexer = Lexer::<StringToken>::new(remain);

        let string = match token.as_ref() {
            StringStart::SingleQuotedStringStart => {
                internal_parse(&mut string_lexer, Delim::SingleQuote)
            }
            StringStart::DoubleQuotedStringStart => {
                internal_parse(&mut string_lexer, Delim::DoubleQuote)
            }
        }
        .map_err(|ParseError { error, location }| ParseError {
            error,
            location: location.translate(token.span),
        })?;

        let string = lexer.strings.add_string(string);

        lexer.set_source_loc(string_lexer.remainder());
        Ok(Some(string))
    }
}

enum Delim {
    SingleQuote,
    DoubleQuote,
}

#[derive(Logos, Debug, PartialEq)]
enum StringToken {
    #[error]
    Error,

    #[regex(br"\r|\n")]
    EndOfLine,
    #[token(br"'")]
    SingleQuote,
    #[token(br#"""#)]
    DoubleQuote,

    #[regex(br#"[^"'\r\n\\]+"#)]
    Plain,

    #[regex(br"\\x[[:xdigit:]][[:xdigit:]]")]
    HexLiteral,
    #[regex(br"\\(?:\d\d\d|\d\d|\d)")]
    DecimalLiteral,

    #[token(br"\\u\{")]
    UnclosedUnicodeEscape,
    #[regex(br"\\u\{[[:xdigit:]]+\}")]
    UnicodeEscape,

    #[regex(br"\\(:?\r\n|\n\r|\r|\n)")]
    LineContinuation,

    #[regex(br"\\z[[:space:]]*")]
    SkipWhitespace,

    #[regex(br"\\[^\d]")]
    UnknownEscapeSequence,
    #[token(br"\a")]
    EscapeSeqBell,
    #[token(br"\b")]
    EscapeSeqBackspace,
    #[token(br"\f")]
    EscapeSeqFormFeed,
    #[token(br"\n")]
    EscapeSeqNewline,
    #[token(br"\r")]
    EscapeSeqCarriageReturn,
    #[token(br"\t")]
    EscapeSeqTab,
    #[token(br"\v")]
    EscapeSeqVerticalTab,
    #[token(br"\\")]
    EscapeSeqBackslash,
    #[token(r#"\""#)]
    EscapeSeqDoubleQuote,
    #[token(br"\'")]
    EscapeSeqSingleQuote,
}

fn internal_parse(
    string_lexer: &mut Lexer<StringToken>,
    delim: Delim,
) -> Result<BString, ParseError> {
    let mut string = BString::default();

    while let Some(token) = string_lexer.next() {
        match token {
            StringToken::Error | StringToken::EndOfLine => {
                return Err(ParseError {
                    error: SyntaxError::UnclosedString,
                    location: string_lexer.span().into(),
                });
            }
            StringToken::SingleQuote => {
                if let Delim::SingleQuote = delim {
                    return Ok(string);
                }
                string.push(b'\'');
            }
            StringToken::DoubleQuote => {
                if let Delim::DoubleQuote = delim {
                    return Ok(string);
                }
                string.push(b'"');
            }
            StringToken::Plain => {
                string.extend_from_slice(string_lexer.slice());
            }
            StringToken::HexLiteral => {
                if let [hex1, hex2] = string_lexer.slice()[..] {
                    let high = char::from(hex1).to_digit(16).expect("Is ascii hex digit") as u8;
                    let low = char::from(hex2).to_digit(16).expect("Is ascii hex digit") as u8;

                    string.push(high * 16 + low);
                } else {
                    unreachable!()
                }
            }
            StringToken::DecimalLiteral => {
                let (hundreds, tens, ones) = match string_lexer.slice()[1..] {
                    [d1, d2, d3] => {
                        let hundreds = char::from(d1).to_digit(10).expect("Is ascii digit");
                        let tens = char::from(d2).to_digit(10).expect("Is ascii digit");
                        let ones = char::from(d3).to_digit(10).expect("Is ascii digit");
                        (hundreds, tens, ones)
                    }
                    [d1, d2] => {
                        let tens = char::from(d1).to_digit(10).expect("Is ascii digit");
                        let ones = char::from(d2).to_digit(10).expect("Is ascii digit");
                        (0, tens, ones)
                    }
                    [d1] => {
                        let ones = char::from(d1).to_digit(10).expect("Is ascii digit");
                        (0, 0, ones)
                    }
                    _ => unreachable!(),
                };
                let result = hundreds * 100 + tens * 10 + ones;
                match u8::try_from(result) {
                    Ok(byte) => string.push(byte),
                    Err(_) => {
                        return Err(ParseError {
                            error: SyntaxError::DecimalEscapeTooLarge,
                            location: string_lexer.span().into(),
                        });
                    }
                }
            }
            StringToken::UnclosedUnicodeEscape => {
                return Err(ParseError {
                    error: SyntaxError::UnclosedUnicodeEscapeSequence,
                    location: string_lexer.span().into(),
                });
            }
            StringToken::UnicodeEscape => {
                let seq = string_lexer.slice();
                match encode_utf8_raw(&seq[3..seq.len() - 1]) {
                    Ok((len, bytes)) => {
                        string.extend_from_slice(&bytes[..len]);
                    }
                    Err(_) => {
                        return Err(ParseError {
                            error: SyntaxError::Utf8ValueTooLarge,
                            location: string_lexer.span().into(),
                        });
                    }
                };
            }
            StringToken::LineContinuation => {
                string.push(b'\n');
            }
            StringToken::UnknownEscapeSequence => {
                return Err(ParseError {
                    error: SyntaxError::InvalidEscapeSequence,
                    location: string_lexer.span().into(),
                });
            }
            StringToken::EscapeSeqBell => {
                string.push(b'\x07');
            }
            StringToken::EscapeSeqBackspace => {
                string.push(b'\x08');
            }
            StringToken::EscapeSeqFormFeed => {
                string.push(b'\x0C');
            }
            StringToken::EscapeSeqNewline => {
                string.push(b'\n');
            }
            StringToken::EscapeSeqCarriageReturn => {
                string.push(b'\r');
            }
            StringToken::EscapeSeqTab => {
                string.push(b'\t');
            }
            StringToken::EscapeSeqVerticalTab => {
                string.push(b'\x0B');
            }
            StringToken::EscapeSeqBackslash => {
                string.push(b'\\');
            }
            StringToken::EscapeSeqDoubleQuote => {
                string.push(b'\"');
            }
            StringToken::EscapeSeqSingleQuote => {
                string.push(b'\'');
            }
            StringToken::SkipWhitespace => {}
        }
    }

    Err(ParseError {
        error: SyntaxError::UnclosedString,
        location: string_lexer.span().into(),
    })
}

/// Encodes a a 4-byte sequence of hex characters into a (potentially invalid -
/// per spec) utf8 byte sequence.
fn encode_utf8_raw(span: &[u8]) -> Result<(usize, [u8; 6]), ()> {
    let (val, _) = u32::from_radix_16_checked(span);
    let val = if let Some(val) = val {
        val
    } else {
        return Err(());
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
        return Err(());
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

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;
    use pretty_assertions::assert_eq;

    use crate::{
        expressions::strings::ConstantString,
        final_parser,
        ASTAllocator,
        StringTable,
    };

    #[test]
    fn single_quoted() -> anyhow::Result<()> {
        let src = r#"'a string'"#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn double_quoted() -> anyhow::Result<()> {
        let src = r#""a string""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_bell() -> anyhow::Result<()> {
        let src = r#""\a""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x07".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_backspace() -> anyhow::Result<()> {
        let src = r#""\b""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x08".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_formfeed() -> anyhow::Result<()> {
        let src = r#""\f""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x0C".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_newline() -> anyhow::Result<()> {
        let src = r#""\n""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\n".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_carriage_return() -> anyhow::Result<()> {
        let src = r#""\r""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\r".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_tab() -> anyhow::Result<()> {
        let src = r#""\t""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\t".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_vertical_tab() -> anyhow::Result<()> {
        let src = r#""\v""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x0B".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_backslash() -> anyhow::Result<()> {
        let src = r#""\\""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\\".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_double_quote() -> anyhow::Result<()> {
        let src = r#""\"""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\"".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_single_quote() -> anyhow::Result<()> {
        let src = r#""\'""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\'".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_z() -> anyhow::Result<()> {
        let src = r#""\z  
            a string""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_z_empty() -> anyhow::Result<()> {
        let src = r#""\za string""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_continuation_newline() -> anyhow::Result<()> {
        let src = "\"line1 \\\n a string\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_continuation_carriage_return() -> anyhow::Result<()> {
        let src = "\"line1 \\\r a string\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_continuation_carriage_return_newline() -> anyhow::Result<()> {
        let src = "\"line1 \\\r\n a string\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_continuation_newline_carriage_return() -> anyhow::Result<()> {
        let src = "\"line1 \\\n\r a string\"";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_unicode() -> anyhow::Result<()> {
        let src = r#""\u{2764}""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some("\u{2764}".as_bytes().as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_unicode_allow_invalid() -> anyhow::Result<()> {
        let src = r#""\u{7FFFFFFF}""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some([253, 191, 191, 191, 191, 191].as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_decimal1() -> anyhow::Result<()> {
        let src = r#""\0""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some("\x00".as_bytes().as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_decimal2() -> anyhow::Result<()> {
        let src = r#""\10""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some("\x0A".as_bytes().as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_decimal3() -> anyhow::Result<()> {
        let src = r#""\102""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some("\x66".as_bytes().as_bstr())
        );

        Ok(())
    }

    #[test]
    fn escape_decimal4() -> anyhow::Result<()> {
        let src = r#""\1029""#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::try_parse)?;

        assert_eq!(result, Some(ConstantString(0)));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some("\x669".as_bytes().as_bstr())
        );

        Ok(())
    }
}
