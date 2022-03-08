use atoi::atoi;
use bstr::BString;
use derive_more::{
    Deref,
    From,
};
use hexf_parse::parse_hexf64;
use logos::{
    Lexer,
    Logos,
};
use strum::Display;

use crate::SourceSpan;

#[cfg(test)]
mod tests;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultilineComment {
    #[strum(to_string = "multiline comment")]
    Valid,
    #[strum(to_string = "unclosed multiline comment")]
    Unclosed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LexedNumber {
    Float(f64),
    Int(i64),
    MalformedNumber,
}

#[derive(Debug, Clone, Copy, PartialEq, Deref, From)]
pub(crate) struct SpannedToken<'src, TokenT = Token> {
    #[deref]
    pub(crate) token: TokenT,
    pub(crate) span: SourceSpan,
    pub(crate) src: &'src [u8],
}

impl<TokenT> AsRef<TokenT> for SpannedToken<'_, TokenT> {
    fn as_ref(&self) -> &TokenT {
        &self.token
    }
}

impl<Token> PartialEq<Token> for SpannedToken<'_, Token>
where
    Token: PartialEq,
{
    fn eq(&self, other: &Token) -> bool {
        self.token == *other
    }
}

#[derive(Logos, Debug, Display, Clone, Copy, PartialEq)]
pub(crate) enum Token {
    #[strum(to_string = "ident")]
    #[regex(br#"[_A-Za-z]\w*"#)]
    Ident,

    #[strum(to_string = "'")]
    #[regex(br#"'"#)]
    SingleQuotedStringStart,

    #[strum(to_string = "\"")]
    #[regex(br#"""#)]
    DoubleQuotedStringStart,

    #[strum(to_string = "multiline string start")]
    #[regex(br#"\[=*\["#, |lex| lex.slice().len() - 2)]
    MultilineStringStart(usize),

    /// Either:
    /// [-] 0x<hex digits>.<hex digits?><power>
    /// [-] 0x<hex digits><power>
    #[strum(to_string = "hexidecimal float")]
    #[regex(
        br#"0[xX][0-9A-Fa-f]+(:?\.[0-9A-Fa-f]*)?[pP][-+]?[0-9A-Fa-f]+"#,
        parse_hex_float
    )]
    HexFloat(LexedNumber),

    /// Exactly:
    /// [-] 0x<hex digits>.<hex digits?>
    #[strum(to_string = "hexidecimal float")]
    #[regex(br#"0[xX][0-9A-Fa-f]+\.[0-9A-Fa-f]*"#, parse_hex_float_no_power)]
    HexFloatNoPower(LexedNumber),

    /// Exactly
    /// [-] 0x<hex digits>
    #[strum(to_string = "hexidecimal integer")]
    #[regex(br#"0[xX][0-9A-Fa-f]+"#, parse_hex_int)]
    HexInt(LexedNumber),

    #[strum(to_string = "float")]
    #[regex(br#"\d+(:?\.\d*(:?[eE][-+]?\d+)?|[eE][-+]?\d+)"#, parse_float)]
    Float(LexedNumber),

    #[strum(to_string = "integer")]
    #[regex(br#"\d+"#, parse_int)]
    Int(LexedNumber),

    #[strum(to_string = "boolean")]
    #[regex(br#"true|false"#, |lex| match lex.slice() {
        b"true" => true,
        b"false" => false,
        _ => unreachable!()
    })]
    Boolean(bool),

    #[strum(to_string = "nil")]
    #[token(b"nil")]
    Nil,

    #[strum(to_string = "whitespace")]
    #[regex(br#"[[:space:]]+"#)]
    Whitespace,

    #[strum(to_string = "comment")]
    #[regex(br#"--(:?\[=*[^\[\r\n]*|[^\[\r\n]*)"#)]
    SinglelineComment,

    /// Stores the length of the opening `=` sequence
    #[regex(br#"--\[=*\["#, parse_multiline_comment)]
    MultilineComment(MultilineComment),

    #[strum(to_string = "and")]
    #[token(b"and")]
    KWand,
    #[strum(to_string = "break")]
    #[token(b"break")]
    KWbreak,
    #[strum(to_string = "do")]
    #[token(b"do")]
    KWdo,
    #[strum(to_string = "else")]
    #[token(b"else")]
    KWelse,
    #[strum(to_string = "elseif")]
    #[token(b"elseif")]
    KWelseif,
    #[strum(to_string = "end")]
    #[token(b"end")]
    KWend,
    #[strum(to_string = "for")]
    #[token(b"for")]
    KWfor,
    #[strum(to_string = "function")]
    #[token(b"function")]
    KWfunction,
    #[strum(to_string = "goto")]
    #[token(b"goto")]
    KWgoto,
    #[strum(to_string = "if")]
    #[token(b"if")]
    KWif,
    #[strum(to_string = "in")]
    #[token(b"in")]
    KWin,
    #[strum(to_string = "local")]
    #[token(b"local")]
    KWlocal,
    #[strum(to_string = "not")]
    #[token(b"not")]
    KWnot,
    #[strum(to_string = "or")]
    #[token(b"or")]
    KWor,
    #[strum(to_string = "repeat")]
    #[token(b"repeat")]
    KWrepeat,
    #[strum(to_string = "return")]
    #[token(b"return")]
    KWreturn,
    #[strum(to_string = "then")]
    #[token(b"then")]
    KWthen,
    #[strum(to_string = "until")]
    #[token(b"until")]
    KWuntil,
    #[strum(to_string = "while")]
    #[token(b"while")]
    KWwhile,

    #[strum(to_string = "[")]
    #[token(b"[")]
    LBracket,
    #[strum(to_string = "]")]
    #[token(b"]")]
    RBracket,

    #[strum(to_string = "{")]
    #[token(b"{")]
    LBrace,
    #[strum(to_string = "}")]
    #[token(b"}")]
    RBrace,

    #[strum(to_string = "(")]
    #[token(b"(")]
    LParen,
    #[strum(to_string = ")")]
    #[token(b")")]
    RParen,

    #[strum(to_string = ":")]
    #[token(b":")]
    Colon,
    #[strum(to_string = "::")]
    #[token(b"::")]
    DoubleColon,
    #[strum(to_string = "+")]
    #[token(b"+")]
    Plus,
    #[strum(to_string = "-")]
    #[token(b"-")]
    Minus,
    #[strum(to_string = "*")]
    #[token(b"*")]
    Star,
    #[strum(to_string = "/")]
    #[token(b"/")]
    Slash,
    #[strum(to_string = "//")]
    #[token(b"//")]
    DoubleSlash,
    #[strum(to_string = "^")]
    #[token(b"^")]
    Caret,
    #[strum(to_string = "%")]
    #[token(b"%")]
    Percent,
    #[strum(to_string = "&")]
    #[token(b"&")]
    Ampersand,
    #[strum(to_string = "~")]
    #[token(b"~")]
    Tilde,
    #[strum(to_string = "|")]
    #[token(b"|")]
    Pipe,
    #[strum(to_string = "<<")]
    #[token(b"<<")]
    DoubleLeftAngle,
    #[strum(to_string = ">>")]
    #[token(b">>")]
    DoubleRightAngle,
    #[strum(to_string = ".")]
    #[token(b".")]
    Period,
    #[strum(to_string = "..")]
    #[token(b"..")]
    DoublePeriod,
    #[strum(to_string = "...")]
    #[token(b"...")]
    Ellipses,
    #[strum(to_string = "<")]
    #[token(b"<")]
    LeftAngle,
    #[strum(to_string = ">")]
    #[token(b">")]
    RightAngle,
    #[strum(to_string = "<=")]
    #[token(b"<=")]
    LeftAngleEquals,
    #[strum(to_string = ">=")]
    #[token(b">=")]
    RightAngleEquals,
    #[strum(to_string = "~=")]
    #[token(b"~=")]
    TildeEquals,
    #[strum(to_string = "#")]
    #[token(b"#")]
    Hashtag,
    #[strum(to_string = ";")]
    #[token(b";")]
    Semicolon,
    #[strum(to_string = ",")]
    #[token(b",")]
    Comma,
    #[strum(to_string = "=")]
    #[token(b"=")]
    Equals,
    #[strum(to_string = "==")]
    #[token(b"==")]
    DoubleEquals,

    #[strum(to_string = "unknown token")]
    #[error]
    Error,
}

impl Token {
    pub(crate) fn is_whitespace(&self) -> bool {
        matches!(
            self,
            Self::Whitespace
                | Self::SinglelineComment
                | Self::MultilineComment(MultilineComment::Valid)
        )
    }
}

impl PartialEq<SpannedToken<'_>> for Token {
    fn eq(&self, other: &SpannedToken) -> bool {
        *self == other.token
    }
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

    let offset = comment_lexer.remainder().as_ptr() as usize - remain.as_ptr() as usize;
    lexer.bump(offset);
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
