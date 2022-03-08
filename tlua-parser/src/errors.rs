use thiserror::Error;

use crate::{
    lexer::Token,
    PeekableLexer,
    SourceSpan,
};

#[derive(Debug, Clone, Copy, Error, PartialEq)]
pub(crate) enum SyntaxError {
    #[error("Expected an expression")]
    ExpectedExpression,
    #[error("Expected a statement")]
    ExpectedStatement,
    #[error("Expected a variable path or function call")]
    ExpectedVarOrCall,
    #[error("Expected a field")]
    ExpectedTableField,
    #[error("Expected an argument list")]
    ExpectedFnArgs,
    #[error("decimal escape too large")]
    DecimalEscapeTooLarge,
    #[error("unclosed string literal")]
    UnclosedString,
    #[error("unrecognized escape sequence")]
    InvalidEscapeSequence,
    #[error("UTF-8 value too large")]
    Utf8ValueTooLarge,
    #[error("Unclosed UTF-8 escape sequence")]
    UnclosedUnicodeEscapeSequence,
    #[error("malformed number")]
    MalformedNumber,
    #[error("expected a variable declaration")]
    ExpectedVariable,
    #[error("expected an identifier or ...")]
    ExpectedIdentOrVaArgs,
    #[error("invalid attribute - expected <const> or <close>")]
    InvalidAttribute,
    #[error("Expected {0:}")]
    ExpectedToken(Token),
    #[error("Expected {0:} or {1:}")]
    ExpectedToken2(Token, Token),
    #[error("Expected {0:}, {1:}, or {2:}")]
    ExpectedToken3(Token, Token, Token),
    #[error("Expected a string")]
    ExpectedString,
    #[allow(unused)]
    #[error("Expected end of file, found: {0:}")]
    ExpectedEOF(Token),
}

#[derive(Debug, Clone, Copy, Error, PartialEq)]
pub struct ParseError {
    pub(crate) error: SyntaxError,
    pub(crate) location: SourceSpan,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {} at {}", self.error, self.location))
    }
}

impl ParseError {
    pub(crate) fn from_here(lexer: &mut PeekableLexer, err: SyntaxError) -> Self {
        Self {
            error: err,
            location: lexer.current_span(),
        }
    }
}

impl<T> From<ParseError> for Result<T, ParseError> {
    fn from(e: ParseError) -> Self {
        Err(e)
    }
}

#[derive(Debug, Error)]
#[error("Errors parsing chunk: {error:#}")]
pub struct ChunkParseError {
    pub error: ParseError,
}

impl From<ParseError> for ChunkParseError {
    fn from(error: ParseError) -> Self {
        ChunkParseError { error }
    }
}

#[cfg(feature = "rendered-errors")]
impl ChunkParseError {
    pub fn build_report(&self) -> ariadne::Report<std::ops::Range<usize>> {
        use ariadne::{
            Label,
            Report,
            ReportKind,
        };

        let (range, label) = (self.error.location, self.error.error);

        Report::build(ReportKind::Error, (), range.start)
            .with_message("Failed to parse LUA")
            .with_label(Label::new(range.start..range.end).with_message(label))
            .finish()
    }
}
