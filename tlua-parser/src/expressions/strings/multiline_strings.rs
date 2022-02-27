use bstr::BString;
use logos::{
    Lexer,
    Logos,
};

use crate::{
    expressions::strings::ConstantString,
    lexer::{
        SpannedToken,
        Token,
    },
    ParseError,
    PeekableLexer,
    SyntaxError,
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

pub(crate) fn parse_string(lexer: &mut PeekableLexer) -> Result<ConstantString, ParseError> {
    let (end_tag_len, base_span) = if let Some(SpannedToken {
        token: Token::MultilineStringStart(tag_len),
        span,
        src: _,
    }) =
        lexer.next_if(|token| matches!(token.as_ref(), Token::MultilineStringStart(_)))
    {
        (tag_len, span)
    } else {
        return Err(ParseError::recoverable_from_here(
            lexer,
            SyntaxError::ExpectedString,
        ));
    };

    let remain = lexer.remainder();
    let mut string_lexer = Lexer::<StringToken>::new(remain);

    let string = internal_parse(&mut string_lexer, end_tag_len).map_err(
        |ParseError {
             error,
             location,
             recoverable,
         }| ParseError {
            error,
            location: location.translate(base_span),
            recoverable,
        },
    )?;

    lexer.set_source_loc(string_lexer.remainder());
    Ok(lexer.strings.add_string(string))
}

fn internal_parse(
    string_lexer: &mut Lexer<StringToken>,
    end_tag_len: usize,
) -> Result<BString, ParseError> {
    let mut string = BString::default();
    let mut first_line = true;

    while let Some(token) = string_lexer.next() {
        match token {
            StringToken::Error => {
                return Err(ParseError {
                    error: SyntaxError::UnclosedString,
                    location: string_lexer.span().into(),
                    recoverable: false,
                });
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

    Err(ParseError {
        error: SyntaxError::UnclosedString,
        location: string_lexer.span().into(),
        recoverable: false,
    })
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use crate::{
        expressions::strings::ConstantString,
        final_parser,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn raw_string() -> anyhow::Result<()> {
        let src = r#"[[]]"#;

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::parse)?;

        assert_eq!(result, ConstantString(0));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"".as_bstr())
        );

        Ok(())
    }

    #[test]
    pub fn raw_string_extended() -> anyhow::Result<()> {
        let src = "[===[]===]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::parse)?;

        assert_eq!(result, ConstantString(0));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"".as_bstr())
        );

        Ok(())
    }

    #[test]
    pub fn raw_string_extended_unbalanced() -> anyhow::Result<()> {
        let src = "[===[]==]===]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::parse)?;

        assert_eq!(result, ConstantString(0));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"]==".as_bstr())
        );

        Ok(())
    }

    #[test]
    pub fn raw_string_first_newline_skip() -> anyhow::Result<()> {
        let src = "[===[\n]===]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::parse)?;

        assert_eq!(result, ConstantString(0));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"".as_bstr())
        );

        Ok(())
    }

    #[test]
    pub fn raw_string_newline_replace() -> anyhow::Result<()> {
        let src = "[===[\nline1\nline2\r\nline3\n\rline4\rline5\n\nline6\r\rline7\r\n\nline8\r\n\rline9\nline10\n\r\r\n\n\rline11]===]";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result =
            final_parser!((src.as_bytes(), &alloc, &mut strings) => ConstantString::parse)?;

        assert_eq!(result, ConstantString(0));
        assert_eq!(
            strings.strings.get_index(0).map(|s| s.as_bstr()),
            Some(b"line1\nline2\nline3\nline4\nline5\n\nline6\n\nline7\n\nline8\n\nline9\nline10\n\n\nline11".as_bstr())
        );

        Ok(())
    }
}
