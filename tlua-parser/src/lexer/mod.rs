use atoi::atoi;
use bstr::{
    BStr,
    BString,
};
use hexf_parse::parse_hexf64;
use indexmap::IndexSet;
use logos::{
    Lexer,
    Logos,
};
use nom::Offset;

use crate::ErrorSpan;

mod multiline_strings;
mod strings;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultilineComment {
    Valid,
    Unclosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LexedString {
    Valid { id: usize },
    Unclosed,
    DecimalEscapeTooLarge { relative_span: ErrorSpan },
    Utf8ValueTooLarge { relative_span: ErrorSpan },
    UnknownEscapeSequence { relative_span: ErrorSpan },
    UnclosedUnicodeSequence { relative_span: ErrorSpan },
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
pub(crate) enum StringToken {
    #[error]
    Error,
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

    #[regex(br#"true|false"#, |lex| match lex.slice() {
        b"true" => true,
        b"false" => false,
        _ => unreachable!()
    })]
    Boolean(bool),

    #[token(b"nil")]
    Nil,

    #[regex(br#"[[:space:]]+"#)]
    Whitespace,

    #[regex(br#"--(:?\[=*[^\[\r\n]*|[^\[\r\n]*)"#)]
    SinglelineComment,

    /// Stores the length of the opening `=` sequence
    #[regex(br#"--\[=*\["#, parse_multiline_comment)]
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

    #[token(b":")]
    Colon,
    #[token(b"::")]
    DoubleColon,
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
    #[token(b".")]
    Period,
    #[token(b"..")]
    DoublePeriod,
    #[token(b"...")]
    Ellipses,
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

#[derive(Logos, Debug, PartialEq)]
enum MultilineCommentToken {
    #[error]
    #[regex(br"[^\]]*", logos::skip)]
    Error,

    #[regex(br"\]=*")]
    PossibleClose,
}

fn parse_multiline_comment(lexer: &mut Lexer<Token>) -> MultilineComment {
    let remain = lexer.remainder();
    // len(--[[) == 4, ignoring any equals tag in between the [[.
    let tag_len = lexer.slice().len() - 4;

    let mut comment_lexer = Lexer::<MultilineCommentToken>::new(remain);

    let token = bump_to_end_of_multiline_comment(&mut comment_lexer, tag_len);

    lexer.bump(remain.offset(comment_lexer.remainder()));
    token
}

fn bump_to_end_of_multiline_comment(
    comment_lexer: &mut Lexer<MultilineCommentToken>,
    open_tag_len: usize,
) -> MultilineComment {
    while let Some(token) = comment_lexer.next() {
        match token {
            MultilineCommentToken::Error => return MultilineComment::Unclosed,
            MultilineCommentToken::PossibleClose => {
                let close_tag_len = comment_lexer.slice().len() - 1;
                if close_tag_len == open_tag_len {
                    if let [b']', ..] = comment_lexer.remainder() {
                        comment_lexer.bump(1);
                        return MultilineComment::Valid;
                    }
                }
            }
        }
    }

    MultilineComment::Unclosed
}

fn parse_multiline_string(lexer: &mut Lexer<Token>) -> LexedString {
    // len([[) == 2, ignoring any equals tag in between the [[.
    let end_tag_len = lexer.slice().len() - 2;
    multiline_strings::parse_string(lexer, end_tag_len)
}

fn parse_string(lexer: &mut Lexer<Token>) -> LexedString {
    strings::parse_string(
        lexer,
        match lexer.slice()[0] {
            b'\'' => strings::Delim::SingleQuote,
            b'\"' => strings::Delim::DoubleQuote,
            _ => unreachable!(),
        },
    )
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
