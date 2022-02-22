use bstr::ByteSlice;
use logos::{
    Lexer,
    Logos,
};
use pretty_assertions::assert_eq;

use crate::lexer::{
    LexedNumber,
    LexedString,
    MultilineComment,
    Token,
};

#[test]
fn test() {
    let src = r#" 
        --3   345   0xff   0xBEBADA
        --3.0     3.1416     314.16e-2     0.31416E1     34e1
        0x1.000000000000000001 0x1 --0xA23p-4   0X1.921FB54442D18P+1
    "#;
    let mut lex = Token::lexer(src.as_bytes());

    while let Some(next) = lex.next() {
        dbg!(next);
    }

    dbg!(lex.extras);

    // assert_eq!(lex.next(), Some(Token::MultilineCommentStart));
    // assert_eq!(lex.slice(), b"====");
}

#[test]
pub fn lexes_empty_short_comment() {
    let src = "--";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_short_comment() {
    let src = "--abcdef";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_short_comments() {
    let src = r#"--abcdef
    --jky"#;

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), Some(Token::Whitespace));
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_empty_long_comment() {
    let src = "--[[]]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Valid))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_long_comment() {
    let src = "--[[abc
            def
        ghi]]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Valid))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_long_comment_invalid() {
    let src = "--[[abc
            def
        ghi]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Unclosed))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_long_comment_tagged() {
    let src = "--[==[abc
            def
        ghi]==]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Valid))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_long_comment_tagged_invalid_short() {
    let src = "--[==[abc
            def
        ghi]=]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Unclosed))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_long_comment_tagged_invalid_long() {
    let src = "--[==[abc
            def
        ghi]===]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Unclosed))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_long_comment_tagged_unbalanced_internal() {
    let src = "--[==[abc
            def
        ghi]===]==]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Valid))
    );
    assert_eq!(lexer.next(), None);
}

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

#[test]
pub fn raw_string() {
    let src = r#"[[]]"#;
    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::MultilineString(LexedString::Valid { id: 0 }))
    );
    assert_eq!(
        lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
        Some(b"".as_bstr())
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn raw_string_extended() {
    let src = "[===[]===]";
    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::MultilineString(LexedString::Valid { id: 0 }))
    );
    assert_eq!(
        lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
        Some(b"".as_bstr())
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn raw_string_extended_unbalanced() {
    let src = "[===[]==]===]";
    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::MultilineString(LexedString::Valid { id: 0 }))
    );
    assert_eq!(
        lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
        Some(b"]==".as_bstr())
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn raw_string_first_newline_skip() {
    let src = "[===[\n]===]";
    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::MultilineString(LexedString::Valid { id: 0 }))
    );
    assert_eq!(
        lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
        Some(b"".as_bstr())
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn raw_string_newline_replace() {
    let src = "[===[\nline1\nline2\r\nline3\n\rline4\rline5\n\nline6\r\rline7\r\n\nline8\r\n\rline9\nline10\n\r\r\n\n\rline11]===]";
    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::MultilineString(LexedString::Valid { id: 0 }))
    );
    assert_eq!(
        lexer.extras.strings.get_index(0).map(|s| s.as_bstr()),
        Some(b"line1\nline2\nline3\nline4\nline5\n\nline6\n\nline7\n\nline8\n\nline9\nline10\n\n\nline11".as_bstr())
    );
    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_float_constant() {
    let src = "1.";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0))));

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_float_constant_decimals() {
    let src = "1.00";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0))));

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_float_constant_exponent() {
    let src = "1.e10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0e10))));

    assert_eq!(lexer.next(), None);
}

#[test]
#[allow(non_snake_case)]
pub fn lexes_float_constant_Exponent() {
    let src = "1.E10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0e10))));

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_float_constant_neg_exponent() {
    let src = "1.e-10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::Float(LexedNumber::Float(1.0e-10)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
#[allow(non_snake_case)]
pub fn lexes_float_constant_neg_Exponent() {
    let src = "1.E-10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::Float(LexedNumber::Float(1.0e-10)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_hex_float_constant() {
    let src = "0x1F.0p-1";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::HexFloat(LexedNumber::Float(15.5)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_integer_constant() {
    let src = "9007199254740993";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::Int(LexedNumber::Int(9007199254740993)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_hex_constant_wrapping() {
    let src = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFFFFFFFFFFF";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::HexInt(LexedNumber::Int(-1152921504606846977)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
pub fn lexes_hex_constant_wrapping_2() {
    let src = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::HexInt(LexedNumber::Int(-1))));

    assert_eq!(lexer.next(), None);
}
