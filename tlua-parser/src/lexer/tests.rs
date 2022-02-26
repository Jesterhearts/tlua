use bstr::ByteSlice;
use logos::{
    Lexer,
    Logos,
};
use pretty_assertions::assert_eq;

use crate::lexer::{
    LexedNumber,
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

    while let Some(token) = lex.next() {
        let span = lex.slice().as_bstr();
        if let Token::Error = token {
            eprintln!("failed to parse: \n{}", span);
            return;
        }
        dbg!((token, span));
    }

    dbg!(lex.extras);

    // assert_eq!(lex.next(), Some(Token::MultilineCommentStart));
    // assert_eq!(lex.slice(), b"====");
}

#[test]
fn lexes_empty_short_comment() {
    let src = "--";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_short_comment() {
    let src = "--abcdef";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_short_comments() {
    let src = r#"--abcdef
    --jky"#;

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), Some(Token::Whitespace));
    assert_eq!(lexer.next(), Some(Token::SinglelineComment));
    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_empty_long_comment() {
    let src = "--[[]]";

    let mut lexer = Lexer::new(src.as_bytes());
    assert_eq!(
        lexer.next(),
        Some(Token::MultilineComment(MultilineComment::Valid))
    );
    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_long_comment() {
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
fn lexes_long_comment_invalid() {
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
fn lexes_long_comment_tagged() {
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
fn lexes_long_comment_tagged_invalid_short() {
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
fn lexes_long_comment_tagged_invalid_long() {
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
fn lexes_long_comment_tagged_unbalanced_internal() {
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
fn lexes_float_constant() {
    let src = "1.";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0))));

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_float_constant_decimals() {
    let src = "1.00";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0))));

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_float_constant_exponent() {
    let src = "1.e10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0e10))));

    assert_eq!(lexer.next(), None);
}

#[test]
#[allow(non_snake_case)]
fn lexes_float_constant_Exponent() {
    let src = "1.E10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Float(LexedNumber::Float(1.0e10))));

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_float_constant_neg_exponent() {
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
fn lexes_float_constant_neg_Exponent() {
    let src = "1.E-10";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::Float(LexedNumber::Float(1.0e-10)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_hex_float_constant() {
    let src = "0x1F.0p-1";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::HexFloat(LexedNumber::Float(15.5)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_integer_constant() {
    let src = "9007199254740993";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::Int(LexedNumber::Int(9007199254740993)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_hex_constant_wrapping() {
    let src = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFFFFFFFFFFF";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(
        lexer.next(),
        Some(Token::HexInt(LexedNumber::Int(-1152921504606846977)))
    );

    assert_eq!(lexer.next(), None);
}

#[test]
fn lexes_hex_constant_wrapping_2() {
    let src = "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::HexInt(LexedNumber::Int(-1))));

    assert_eq!(lexer.next(), None);
}

#[test]
fn parses_nil() {
    let src = "nil";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Nil));

    assert_eq!(lexer.next(), None);
}

#[test]
fn parses_true() {
    let src = "true";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Boolean(true)));

    assert_eq!(lexer.next(), None);
}

#[test]
fn parses_false() {
    let src = "false";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Boolean(false)));

    assert_eq!(lexer.next(), None);
}

#[test]
fn parses_ident() {
    let src = "_";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Ident));
    assert_eq!(lexer.slice(), b"_");
    assert_eq!(lexer.next(), None);
}

#[test]
fn parses_ident_alpha_start() {
    let src = "a";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Ident));
    assert_eq!(lexer.slice(), b"a");
    assert_eq!(lexer.next(), None);
}

#[test]
fn parses_ident_alphanum() {
    let src = "_abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_";

    let mut lexer = Lexer::new(src.as_bytes());

    assert_eq!(lexer.next(), Some(Token::Ident));
    assert_eq!(lexer.slice(), src.as_bytes());
    assert_eq!(lexer.next(), None);
}
