use atoi::{
    atoi,
    FromRadix16Checked,
};
use bstr::{
    BStr,
    BString,
    ByteSlice,
};
use hexf_parse::parse_hexf64;
use indexmap::IndexSet;
use logos::{
    Lexer,
    Logos,
};
use nom::Offset;

#[cfg(test)]
mod tests;

const LUA_WHITESPACE: &[u8] = b"\n\r\t\x0B\x0C ";
const HEX_DIGITS: &[u8] = b"0123456789abcdefABCDEF";

macro_rules! any_digit {
    () => {
        b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9'
    };
}

macro_rules! any_hex_digit {
    () => {
        any_digit!()
            | b'a'
            | b'b'
            | b'c'
            | b'd'
            | b'e'
            | b'f'
            | b'A'
            | b'B'
            | b'C'
            | b'D'
            | b'E'
            | b'F'
    };
}

struct NoMatchingBracket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultilineComment {
    Valid,
    Unclosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexedString {
    Valid { id: usize },
    Unclosed,
    DecimalEscapeTooLarge,
    Utf8ValueTooLarge,
    UnknownEscapeSequence,
    UnclosedUnicodeSequence,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LexedNumber {
    Float(f64),
    Int(i64),
    MalformedNumber,
}

#[derive(Debug, Default)]
pub struct StringTable {
    idents: IndexSet<BString>,
    strings: IndexSet<BString>,
}

impl StringTable {
    fn add_ident<'s>(&mut self, ident: impl Into<&'s BStr> + Copy) -> usize {
        if let Some(id) = self.idents.get_index_of(ident.into()) {
            id
        } else {
            self.idents.insert_full(ident.into().to_owned()).0
        }
    }

    fn add_string(&mut self, string: BString) -> usize {
        self.strings.insert_full(string).0
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(extras = StringTable)]
pub(crate) enum Token {
    #[regex(br#"[_A-Za-z]\w*"#, |lex| lex.extras.add_ident(lex.slice()))]
    Ident(usize),

    #[regex(br#"["']"#, parse_string)]
    String(LexedString),

    #[regex(br#"\[=*\["#, parse_multiline_string)]
    MultilineString(LexedString),

    /// Either:
    /// [-] 0x<hex digits>.<hex digits?><power>
    /// [-] 0x<hex digits><power>
    #[regex(
        br#"-?0[xX][0-9A-Fa-f]+(:?\.[0-9A-Fa-f]*)?[pP][-+]?[0-9A-Fa-f]+"#,
        parse_hex_float
    )]
    HexFloat(LexedNumber),

    /// Exactly:
    /// [-] 0x<hex digits>.<hex digits?>
    #[regex(br#"-?0[xX][0-9A-Fa-f]+(:?\.[0-9A-Fa-f]*)"#, parse_hex_float_no_power)]
    HexFloatNoPower(LexedNumber),

    /// Exactly
    /// [-] 0x<hex digits>
    #[regex(br#"-?0[xX][0-9A-Fa-f]+"#, parse_hex_int)]
    HexInt(LexedNumber),

    #[regex(br#"-?\d+(:?\.\d*(:?[eE][-+]?\d+)?|[eE][-+]?\d+)"#, parse_float)]
    Float(LexedNumber),

    #[regex(br#"-?\d+"#, parse_int)]
    Int(LexedNumber),

    #[regex(br#"[\n\r\t\x0B\x0C ]+"#)]
    Whitespace,

    #[regex(b"--", bump_to_end_of_singleline_comment)]
    SinglelineComment,

    /// Stores the length of the opening `=` sequence
    #[regex(br#"--\[=*\["#, bump_to_end_of_multiline_comment)]
    MultilineComment(MultilineComment),

    #[token(b"and")]
    KWand,
    #[token(b"break")]
    KWbreak,
    #[token(b"do")]
    KWdo,
    #[token(b"else")]
    KWelse,
    #[token(b"elseif")]
    KWelseif,
    #[token(b"end")]
    KWend,
    #[token(b"false")]
    KWfalse,
    #[token(b"for")]
    KWfor,
    #[token(b"function")]
    KWfunction,
    #[token(b"goto")]
    KWgoto,
    #[token(b"if")]
    KWif,
    #[token(b"in")]
    KWin,
    #[token(b"local")]
    KWlocal,
    #[token(b"nil")]
    KWnil,
    #[token(b"not")]
    KWnot,
    #[token(b"or")]
    KWor,
    #[token(b"repeat")]
    KWrepeat,
    #[token(b"return")]
    KWreturn,
    #[token(b"then")]
    KWthen,
    #[token(b"true")]
    KWtrue,
    #[token(b"until")]
    KWuntil,
    #[token(b"while")]
    KWwhile,

    #[token(b"[")]
    LBracket,
    #[token(b"]")]
    RBracket,

    #[token(b"{")]
    LBrace,
    #[token(b"}")]
    RBrace,

    #[token(b"(")]
    LParen,
    #[token(b")")]
    RParen,

    #[token(b"::")]
    LabelMark,

    #[token(b"...")]
    Ellipses,
    #[token(b"+")]
    Plus,
    #[token(b"-")]
    Minus,
    #[token(b"*")]
    Star,
    #[token(b"/")]
    Slash,
    #[token(b"//")]
    DoubleSlash,
    #[token(b"^")]
    Caret,
    #[token(b"%")]
    Percent,
    #[token(b"&")]
    Ampersand,
    #[token(b"~")]
    Tilde,
    #[token(b"|")]
    Pipe,
    #[token(b"<<")]
    DoubleLeftAngle,
    #[token(b">>")]
    DoubleRightAngle,
    #[token(b"..")]
    DoublePeriod,
    #[token(b"<")]
    LeftAngle,
    #[token(b">")]
    RightAngle,
    #[token(b"<=")]
    LeftAngleEquals,
    #[token(b">=")]
    RightAngleEquals,
    #[token(b"~=")]
    TildeEquals,
    #[token(b"#")]
    Hashtag,
    #[token(b";")]
    Semicolon,
    #[token(b",")]
    Comma,
    #[token(b"=")]
    Equals,
    #[token(b"==")]
    DoubleEquals,

    #[error]
    Error,
}

fn bump_to_end_of_singleline_comment(lexer: &mut Lexer<Token>) {
    let remain = lexer.remainder();
    let line_len = remain.find_byteset(b"\r\n").unwrap_or_else(|| remain.len());
    lexer.bump(line_len);
}

fn bump_to_matching_long_bracket<'f>(
    lexer: &'f mut Lexer<Token>,
    tag_len: usize,
) -> Result<&'f [u8], NoMatchingBracket> {
    let initial_remain = lexer.remainder();
    let mut remain = initial_remain;

    if tag_len == 0 {
        if let Some(end_of_span) = remain.find(b"]]") {
            lexer.bump(end_of_span);
            // Eat ']]'
            lexer.bump(2);
            return Ok(&initial_remain[..end_of_span]);
        }
    } else {
        while let Some(rbracket_loc) = remain.find_byte(b']') {
            let tag_start = rbracket_loc + 1;
            remain = &remain[tag_start..];

            if let Some(close_len) = remain.find_not_byteset(b"=") {
                remain = &remain[close_len..];
                if close_len == tag_len && remain.starts_with(b"]") {
                    lexer.bump(initial_remain.offset(remain));
                    // Eat ']'
                    lexer.bump(1);

                    let span_length = initial_remain.offset(remain) - tag_len - 1;
                    return Ok(&initial_remain[..span_length]);
                }
            } else {
                break;
            }
        }
    }

    lexer.bump(initial_remain.len());
    Err(NoMatchingBracket)
}

fn bump_to_end_of_multiline_comment(lexer: &mut Lexer<Token>) -> MultilineComment {
    let span = lexer.slice();
    // len(--[[) == 4, ignoring any equals tag in between the [[.
    let tag_len = span.len() - 4;

    bump_to_matching_long_bracket(lexer, tag_len)
        .map_or(MultilineComment::Unclosed, |_| MultilineComment::Valid)
}

fn parse_multiline_string(lexer: &mut Lexer<Token>) -> LexedString {
    let span = lexer.slice();
    // len([[) == 2, ignoring any equals tag in between the [[.
    let tag_len = span.len() - 2;

    let mut span = match bump_to_matching_long_bracket(lexer, tag_len) {
        Ok(span) => span,
        Err(NoMatchingBracket) => return LexedString::Unclosed,
    };

    let mut string = BString::default();

    span = match span {
        [b'\r', b'\n', rest @ ..] | [b'\n', b'\r', rest @ ..] => rest,
        [b'\r' | b'\n', rest @ ..] => rest,
        rest => rest,
    };

    while let Some(cr_nl) = span.find_byteset(b"\r\n") {
        let (data, trailing) = span.split_at(cr_nl);
        string.extend_from_slice(data);
        string.push(b'\n');

        span = match trailing {
            [b'\r', b'\n', rest @ ..] | [b'\n', b'\r', rest @ ..] => rest,
            [b'\r' | b'\n', rest @ ..] => rest,
            rest => rest,
        };
    }

    string.extend_from_slice(span);
    LexedString::Valid {
        id: lexer.extras.add_string(string),
    }
}

fn parse_string(lexer: &mut Lexer<Token>) -> LexedString {
    let delim = lexer.slice()[0];
    let initial_remain = lexer.remainder();

    let mut string = Ok(BString::default());
    let mut remain = initial_remain;

    while let Some(delim_or_escape) = remain.find_byteset([delim, b'\\', b'\r', b'\n']) {
        let (data, trailing) = remain.split_at(delim_or_escape);
        if let Ok(string) = string.as_mut() {
            string.extend_from_slice(data);
        }

        // Skip the delimiter
        remain = &trailing[1..];

        let delim_or_escape = trailing[0];
        if delim_or_escape == delim {
            lexer.bump(initial_remain.offset(remain));
            return string.map_or_else(
                |e| e,
                |string| LexedString::Valid {
                    id: lexer.extras.add_string(string),
                },
            );
        }

        if let b'\r' | b'\n' = delim_or_escape {
            lexer.bump(initial_remain.offset(remain));
            return string.err().unwrap_or(LexedString::Unclosed);
        }

        match remain {
            [b'\n', b'\r', rest @ ..] | [b'\r', b'\n', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\n');
                }
                remain = rest;
                continue;
            }
            [b'\n' | b'\r', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\n');
                }
                remain = rest;
                continue;
            }

            [b'a', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\x07');
                }
                remain = rest;
            }
            [b'b', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\x08');
                }
                remain = rest;
            }
            [b'f', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\x0C');
                }
                remain = rest;
            }
            [b'n', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\n');
                }
                remain = rest;
            }
            [b'r', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\r');
                }
                remain = rest;
            }
            [b't', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\t');
                }
                remain = rest;
            }
            [b'v', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\x0B');
                }
                remain = rest;
            }
            [b'\\', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\\');
                }
                remain = rest;
            }
            [b'"', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'"');
                }
                remain = rest;
            }
            [b'\'', rest @ ..] => {
                if let Ok(string) = string.as_mut() {
                    string.push(b'\'');
                }
                remain = rest;
            }
            [b'z', rest @ ..] => {
                let first_non_ws = if let Some(idx) = rest.find_not_byteset(LUA_WHITESPACE) {
                    idx
                } else {
                    return string.err().unwrap_or(LexedString::Unclosed);
                };

                remain = &rest[first_non_ws..];
            }
            [b'u', b'{', rest @ ..] => {
                let hex_digits = if let Some(end) = rest.find_not_byteset(HEX_DIGITS) {
                    end
                } else {
                    return string.err().unwrap_or(LexedString::UnclosedUnicodeSequence);
                };

                let (span, rest) = rest.split_at(hex_digits);
                if rest[0] != b'}' {
                    remain = rest;
                    string = Err(LexedString::UnclosedUnicodeSequence);
                } else {
                    remain = &rest[1..];

                    match (&mut string, encode_utf8_raw(span)) {
                        (Ok(string), Ok((len, data))) => {
                            string.extend_from_slice(&data[..len]);
                        }
                        (string @ Ok(_), Err(e)) => {
                            *string = Err(e);
                        }
                        (Err(_), _) => {}
                    };
                }
            }
            [b'x', hex1 @ any_hex_digit!(), hex2 @ any_hex_digit!(), rest @ ..] => {
                let high = char::from(*hex1).to_digit(16).expect("Is ascii hex digit") as u8;
                let low = char::from(*hex2).to_digit(16).expect("Is ascii hex digit") as u8;
                remain = rest;

                if let Ok(string) = string.as_mut() {
                    string.push(high * 16 + low);
                }
            }
            [d1 @ any_digit!(), d2 @ any_digit!(), d3 @ any_digit!(), rest @ ..] => {
                let hundreds = char::from(*d1).to_digit(10).expect("Is ascii digit");
                let tens = char::from(*d2).to_digit(10).expect("Is ascii digit");
                let ones = char::from(*d3).to_digit(10).expect("Is ascii digit");
                remain = rest;

                let result = hundreds * 100 + tens * 10 + ones;
                match (&mut string, u8::try_from(result)) {
                    (Ok(string), Ok(byte)) => {
                        string.push(byte);
                    }
                    (string @ Ok(_), Err(_)) => {
                        *string = Err(LexedString::DecimalEscapeTooLarge);
                    }
                    (Err(_), _) => {}
                }
            }
            _ => {
                lexer.bump(initial_remain.offset(remain));
                return LexedString::UnknownEscapeSequence;
            }
        }
    }

    string.err().unwrap_or(LexedString::Unclosed)
}

/// Encodes a a 4-byte sequence of hex characters into a (potentially invalid -
/// per spec) utf8 byte sequence.
fn encode_utf8_raw(span: &[u8]) -> Result<(usize, [u8; 6]), LexedString> {
    let (val, _) = u32::from_radix_16_checked(span);
    let val = if let Some(val) = val {
        val
    } else {
        return Err(LexedString::Utf8ValueTooLarge);
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
        return Err(LexedString::Utf8ValueTooLarge);
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

fn parse_float(lexer: &mut Lexer<Token>) -> LexedNumber {
    let span = lexer.slice();

    // SAFETY: We know that we only have <digits>/eE/-+
    unsafe { std::str::from_utf8_unchecked(span) }
        .parse()
        .map(LexedNumber::Float)
        .unwrap_or(LexedNumber::MalformedNumber)
}

fn parse_hex_float(lexer: &mut Lexer<Token>) -> LexedNumber {
    let span = lexer.slice();

    // SAFETY: We know that we only have 0x/<hex digits>/pP/-+
    parse_hexf64(unsafe { std::str::from_utf8_unchecked(span) }, false)
        .map(LexedNumber::Float)
        .unwrap_or(LexedNumber::MalformedNumber)
}

fn parse_hex_float_no_power(lexer: &mut Lexer<Token>) -> LexedNumber {
    let span = lexer.slice();
    //TODO(lang-5.4): `hexf` won't parse 0x1E (or anything missing a trailing
    // 'pXX') So we convert to an actual string and add p0 at the end.
    //
    // It'll work mostly probably for now...
    //
    // The source code is also CC-0, which means forking it to address these
    // issues is kind of weird for this project's license.
    let mut string = BString::from(span);
    string.extend_from_slice(b"p0");

    // SAFETY: We know that we only have 0x/<hex digits>/pP/-+
    parse_hexf64(unsafe { std::str::from_utf8_unchecked(&string) }, false)
        .map(LexedNumber::Float)
        .unwrap_or(LexedNumber::MalformedNumber)
}

fn parse_int(lexer: &mut Lexer<Token>) -> LexedNumber {
    let span = lexer.slice();
    atoi(span)
        .map(LexedNumber::Int)
        .unwrap_or_else(|| parse_float(lexer))
}

fn parse_hex_int(lexer: &mut Lexer<Token>) -> LexedNumber {
    let digits = lexer.slice();
    let is_negative = digits[0] == b'-';
    let start = if is_negative {
        3 // -0x
    } else {
        2 // 0x
    };

    let mut result: i64 = 0;
    for digit in digits.iter().skip(start).copied().map(|d| match d {
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

    LexedNumber::Int(result)
}
