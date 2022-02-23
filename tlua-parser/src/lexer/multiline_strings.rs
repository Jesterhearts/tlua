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

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum StringToken {
    #[error]
    Error,

    #[regex(br"\r\n|\n\r|\r|\n")]
    Eol,

    #[regex(br#"[^\]\r\n]*"#)]
    Plain,

    #[regex(br#"]=*"#)]
    PossibleClose,
}

pub(crate) fn parse_string(lexer: &mut Lexer<Token>, end_tag_len: usize) -> LexedString {
    let remain = lexer.remainder();
    let mut string_lexer = Lexer::<StringToken>::new(remain);

    let string =
        match internal_parse(&mut string_lexer, end_tag_len).map(|string| LexedString::Valid {
            id: lexer.extras.add_string(string),
        }) {
            Ok(s) | Err(s) => s,
        };

    lexer.bump(remain.offset(string_lexer.remainder()));
    string
}

fn internal_parse(
    string_lexer: &mut Lexer<StringToken>,
    end_tag_len: usize,
) -> Result<BString, LexedString> {
    let mut string = BString::default();
    let mut first_line = true;

    while let Some(token) = string_lexer.next() {
        match token {
            StringToken::Error => {
                return Err(LexedString::Unclosed);
            }
            StringToken::Eol => {
                if !first_line {
                    string.push(b'\n');
                }
            }
            StringToken::Plain => {
                string.extend_from_slice(string_lexer.slice());
            }
            StringToken::PossibleClose => {
                let len = string_lexer.slice().len() - 1;
                if len == end_tag_len {
                    if let [b']', ..] = string_lexer.remainder() {
                        string_lexer.bump(1);
                        return Ok(string);
                    }
                }
                string.extend_from_slice(string_lexer.slice());
            }
        }
        first_line = false;
    }

    Err(LexedString::Unclosed)
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
}
