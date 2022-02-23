use atoi::FromRadix16Checked;
use bstr::BString;
use logos::{
    Lexer,
    Logos,
};
use nom::Offset;

use crate::lexer::{
    LexedString,
    Token,
};

pub(crate) enum Delim {
    SingleQuote,
    DoubleQuote,
}

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum StringToken {
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
    #[regex(br"\\\d\d\d")]
    DecimalLiteral,

    #[token(br"\\u\{")]
    UnclosedUnicodeEscape,
    #[regex(br"\\u\{[[:xdigit:]]+\}")]
    UnicodeEscape,

    #[regex(br"\\(:?\r\n|\n\r|\r|\n)")]
    LineContinuation,

    #[regex(br"\\z[[:space:]]*")]
    SkipWhitespace,

    #[regex(br"\\.")]
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

pub(crate) fn parse_string(lexer: &mut Lexer<Token>, delim: Delim) -> LexedString {
    let remain = lexer.remainder();
    let mut string_lexer = Lexer::<StringToken>::new(remain);

    let string = match internal_parse(&mut string_lexer, delim).map(|string| LexedString::Valid {
        id: lexer.extras.add_string(string),
    }) {
        Ok(s) | Err(s) => s,
    };

    lexer.bump(remain.offset(string_lexer.remainder()));
    string
}

fn internal_parse(
    string_lexer: &mut Lexer<StringToken>,
    delim: Delim,
) -> Result<BString, LexedString> {
    let mut error = LexedString::Unclosed;
    let mut string = BString::default();

    while let Some(token) = string_lexer.next() {
        match token {
            StringToken::Error | StringToken::EndOfLine => {
                return Err(LexedString::Unclosed);
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
                if let [d1, d2, d3] = string_lexer.slice()[..] {
                    let hundreds = char::from(d1).to_digit(10).expect("Is ascii digit");
                    let tens = char::from(d2).to_digit(10).expect("Is ascii digit");
                    let ones = char::from(d3).to_digit(10).expect("Is ascii digit");

                    let result = hundreds * 100 + tens * 10 + ones;
                    match u8::try_from(result) {
                        Ok(byte) => string.push(byte),
                        Err(_) => {
                            error = LexedString::DecimalEscapeTooLarge {
                                relative_span: string_lexer.span().into(),
                            };
                            break;
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            StringToken::UnclosedUnicodeEscape => {
                error = LexedString::UnclosedUnicodeSequence {
                    relative_span: string_lexer.span().into(),
                };
                break;
            }
            StringToken::UnicodeEscape => {
                let seq = string_lexer.slice();
                match encode_utf8_raw(&seq[3..seq.len() - 1]) {
                    Ok((len, bytes)) => {
                        string.extend_from_slice(&bytes[..len]);
                    }
                    Err(_) => {
                        error = LexedString::Utf8ValueTooLarge {
                            relative_span: string_lexer.span().into(),
                        };
                        break;
                    }
                };
            }
            StringToken::LineContinuation => {
                string.push(b'\n');
            }
            StringToken::UnknownEscapeSequence => {
                error = LexedString::UnknownEscapeSequence {
                    relative_span: string_lexer.span().into(),
                };
                break;
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

    for token in string_lexer {
        match token {
            StringToken::Error | StringToken::EndOfLine => {
                break;
            }
            StringToken::SingleQuote => {
                if let Delim::SingleQuote = delim {
                    break;
                }
            }
            StringToken::DoubleQuote => {
                if let Delim::DoubleQuote = delim {
                    break;
                }
            }
            _ => {}
        }
    }

    Err(error)
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
    use logos::Lexer;

    use crate::lexer::{
        LexedString,
        Token,
    };

    #[test]
    pub fn single_quoted() {
        let src = r#"'a string'"#;

        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn double_quoted() {
        let src = r#""a string""#;

        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_bell() {
        let src = r#""\a""#;

        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x07".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_backspace() {
        let src = r#""\b""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x08".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_formfeed() {
        let src = r#""\f""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x0C".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_newline() {
        let src = r#""\n""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\n".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_carriage_return() {
        let src = r#""\r""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\r".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_tab() {
        let src = r#""\t""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\t".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_vertical_tab() {
        let src = r#""\v""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\x0B".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_backslash() {
        let src = r#""\\""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\\".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_double_quote() {
        let src = r#""\"""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\"".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_single_quote() {
        let src = r#""\'""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"\'".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_z() {
        let src = r#""\z  
            a string""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_z_empty() {
        let src = r#""\za string""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_continuation_newline() {
        let src = "\"line1 \\\n a string\"";
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_continuation_carriage_return() {
        let src = "\"line1 \\\r a string\"";
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_continuation_carriage_return_newline() {
        let src = "\"line1 \\\r\n a string\"";
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_continuation_newline_carriage_return() {
        let src = "\"line1 \\\n\r a string\"";
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1 \n a string".as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_unicode() {
        let src = r#""\u{2764}""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some("\u{2764}".as_bytes().as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }

    #[test]
    pub fn escape_unicode_allow_invalid() {
        let src = r#""\u{7FFFFFFF}""#;
        let mut lexer = Lexer::new(src.as_bytes());

        assert_eq!(
            lexer.next(),
            Some(Token::String(LexedString::Valid { id: 0 }))
        );
        assert_eq!(
            lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
            Some([253, 191, 191, 191, 191, 191].as_bstr())
        );
        assert_eq!(lexer.next(), None);
    }
}
