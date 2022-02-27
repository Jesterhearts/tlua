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
    #[error("Expected a prefix expression")]
    ExpectedPrefixExpression,
    #[error("Expected an argument list")]
    ExpectedFnArgs,
    #[error("Expected a function definition")]
    ExpectedFunctionDef,
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
    #[error("Expected end of file, found: {0:}")]
    ExpectedEOF(Token),
    #[error("Expected {0:}")]
    ExpectedToken(Token),
    #[error("Expected a string")]
    ExpectedString,
}

#[derive(Debug, Clone, Copy, Error, PartialEq)]
pub struct ParseError {
    pub(crate) error: SyntaxError,
    pub(crate) location: SourceSpan,
    pub(crate) recoverable: bool,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {} at {}", self.error, self.location))
    }
}

impl ParseError {
    pub(crate) fn recoverable_from_here(lexer: &mut PeekableLexer, err: SyntaxError) -> Self {
        Self {
            error: err,
            location: lexer.current_span(),
            recoverable: true,
        }
    }

    pub(crate) fn unrecoverable_from_here(lexer: &mut PeekableLexer, err: SyntaxError) -> Self {
        Self {
            error: err,
            location: lexer.current_span(),
            recoverable: false,
        }
    }

    pub(crate) fn recover_with<T, F: FnOnce() -> Result<T, Self>>(
        self,
        recover: F,
    ) -> Result<T, Self> {
        if self.recoverable {
            let res = recover();
            res.map_err(|e_new| {
                if !e_new.recoverable || e_new.location.end > self.location.end {
                    e_new
                } else {
                    self
                }
            })
        } else {
            Err(self)
        }
    }

    pub(crate) fn map_recoverable_err<F: FnOnce() -> Self>(self, err: F) -> Self {
        if self.recoverable {
            err()
        } else {
            self
        }
    }
}

impl<T> From<ParseError> for Result<T, ParseError> {
    fn from(e: ParseError) -> Self {
        Err(e)
    }
}
pub(crate) trait ParseErrorExt: Sized {
    type Data;
    type Error;

    fn mark_unrecoverable(self) -> Result<Self::Data, Self::Error>;

    fn recover(self) -> Result<Option<Self::Data>, Self::Error>;
    fn recover_with<F: FnOnce() -> Result<Self::Data, Self::Error>>(
        self,
        recover: F,
    ) -> Result<Self::Data, Self::Error>;

    #[allow(clippy::type_complexity)]
    fn chain_or_recover_with<D, F: FnOnce() -> Result<D, Self::Error>>(
        self,
        next: F,
    ) -> Result<(Option<Self::Data>, D), Self::Error>;

    fn ok_or_else<F: FnOnce() -> Self::Error>(self, err: F) -> Result<Self::Data, Self::Error>;
}

impl<T> ParseErrorExt for Result<T, ParseError> {
    type Data = T;
    type Error = ParseError;

    #[inline]
    fn mark_unrecoverable(self) -> Result<Self::Data, Self::Error> {
        self.map_err(|mut e| {
            e.recoverable = false;
            e
        })
    }

    #[inline]
    fn recover(self) -> Result<Option<Self::Data>, Self::Error> {
        match self {
            Ok(data) => Ok(Some(data)),
            Err(e) => e.recover_with(|| Ok(None)),
        }
    }

    #[inline]
    fn recover_with<F: FnOnce() -> Result<Self::Data, Self::Error>>(
        self,
        recover: F,
    ) -> Result<Self::Data, Self::Error> {
        match self {
            ok @ Ok(_) => ok,
            Err(e) => e.recover_with(recover),
        }
    }

    fn chain_or_recover_with<D, F: FnOnce() -> Result<D, Self::Error>>(
        self,
        next: F,
    ) -> Result<(Option<Self::Data>, D), Self::Error> {
        match self {
            Ok(v) => match next() {
                Ok(d) => Ok((Some(v), d)),
                Err(e) => Err(e),
            },
            Err(e) => e.recover_with(next).map(|d| (None, d)),
        }
    }

    fn ok_or_else<F: FnOnce() -> Self::Error>(self, err: F) -> Result<Self::Data, Self::Error> {
        match self {
            ok @ Ok(_) => ok,
            Err(e) => Err(e.map_recoverable_err(err)),
        }
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
