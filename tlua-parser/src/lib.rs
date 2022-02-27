use bstr::{
    BStr,
    BString,
};
use bumpalo::Bump;
use indexmap::IndexSet;
use logos::Lexer;
use nom::Offset;
use thiserror::Error;
use tlua_strings::LuaString;

pub mod block;
pub mod expressions;
pub mod identifiers;
mod lexer;
pub mod list;
pub mod prefix_expression;
pub mod statement;

use crate::{
    block::Block,
    expressions::strings::ConstantString,
    identifiers::Ident,
    lexer::{
        SpannedToken,
        Token,
    },
    list::List,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    start: usize,
    end: usize,
}

impl From<logos::Span> for SourceSpan {
    fn from(span: logos::Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl std::fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "bytes: {start}..{end}",
            start = self.start,
            end = self.end
        ))
    }
}

impl SourceSpan {
    /// Relocate this span to be relative to a `base` span.
    pub(crate) fn translate(&self, base: Self) -> Self {
        let SourceSpan { start, end } = self;
        Self {
            start: base.start + start,
            end: base.start + end,
        }
    }
}

#[derive(Debug, Clone, Copy, Error, PartialEq)]
pub struct ParseError {
    error: SyntaxError,
    location: SourceSpan,
    recoverable: bool,
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

pub(crate) struct SpannedTokenStream<'src, 'strings> {
    src: &'src [u8],
    lexer: Lexer<'src, Token>,
    peeked: Option<SpannedToken<'src>>,
    pub(crate) strings: &'strings mut StringTable,
}

impl SpannedTokenStream<'_, '_> {
    fn new<'src, 'strings>(
        src: &'src [u8],
        strings: &'strings mut StringTable,
    ) -> SpannedTokenStream<'src, 'strings> {
        SpannedTokenStream {
            src,
            lexer: Lexer::new(src),
            peeked: None,
            strings,
        }
    }
}

impl<'src> SpannedTokenStream<'src, '_> {
    fn next(&mut self) -> Option<SpannedToken<'src>> {
        self.peeked.take().or_else(|| {
            while let Some(token) = self.lexer.next() {
                if !Token::is_whitespace(&token) {
                    return Some(SpannedToken {
                        token,
                        span: self.lexer.span().into(),
                        src: self.lexer.slice(),
                    });
                }
            }
            None
        })
    }

    fn current_span(&mut self) -> SourceSpan {
        self.peek()
            .map(|tok| tok.span)
            .unwrap_or_else(|| SourceSpan {
                start: self.src.len(),
                end: self.src.len(),
            })
    }

    fn peek(&mut self) -> Option<SpannedToken<'src>> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked
    }

    fn next_if(
        &mut self,
        filter: impl FnOnce(&SpannedToken<'src>) -> bool,
    ) -> Option<SpannedToken<'src>> {
        if self.peek().filter(filter).is_some() {
            self.peeked.take()
        } else {
            None
        }
    }

    fn next_if_eq(
        &mut self,
        token: impl PartialEq<SpannedToken<'src>>,
    ) -> Option<SpannedToken<'src>> {
        self.next_if(|t| token == *t)
    }

    fn remainder(&self) -> &'src [u8] {
        if let Some(peeked) = self.peeked.as_ref() {
            &self.src[peeked.span.start..]
        } else {
            self.lexer.remainder()
        }
    }

    fn reset(&mut self, token: SpannedToken<'src>) {
        self.peeked = Some(token);
        self.lexer = Lexer::new(self.src);
        self.lexer
            .bump(self.src.offset(token.src) + token.src.len());
    }

    fn set_source_loc(&mut self, src: &'src [u8]) {
        self.peeked = None;
        self.lexer = Lexer::new(self.src);
        self.lexer.bump(self.src.offset(src));
        let _peek = self.peek();
    }
}

type PeekableLexer<'src, 'strings> = SpannedTokenStream<'src, 'strings>;

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

macro_rules! final_parser {
    (($input:expr, $alloc:expr, $strings:expr) => $parser:expr) => {{
        let mut token_stream = $crate::SpannedTokenStream::new($input, $strings);
        let res = $parser(&mut token_stream, $alloc).and_then(|val| match token_stream.peek() {
            None => Ok(val),
            Some(token) => Err($crate::ParseError {
                error: $crate::SyntaxError::ExpectedEOF(token.token),
                location: token.span,
                recoverable: false,
            }),
        });

        res
    }};
}

pub(crate) use final_parser;

pub(crate) fn parse_list1_split_tail<'chunk, 'src, P, O>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<(List<'chunk, O>, O), ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    O: 'chunk,
{
    let head = parser(lexer, alloc)?;

    let mut list = List::default();
    let mut current = list.cursor_mut();

    let mut prev = head;
    loop {
        if let Some(next) = parser(lexer, alloc).recover()? {
            current = current.alloc_insert_advance(alloc, prev);
            prev = next;
        } else {
            return Ok((list, prev));
        }
    }
}

pub(crate) fn parse_list_with_head<'chunk, 'src, P, O>(
    head: O,
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    O: 'chunk,
{
    let mut list = List::default();
    let mut current = list.cursor_mut();
    current = current.alloc_insert_advance(alloc, head);

    while let Some(next) = parser(lexer, alloc).recover()? {
        current = current.alloc_insert_advance(alloc, next);
    }

    Ok(list)
}

pub(crate) fn parse_list1<'chunk, 'src, P, O>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    O: 'chunk,
{
    let head = parser(lexer, alloc)?;
    parse_list_with_head(head, lexer, alloc, parser)
}

pub(crate) fn parse_separated_list1<'chunk, 'src, P, M, O>(
    lexer: &mut PeekableLexer<'src, '_>,
    alloc: &'chunk ASTAllocator,
    parser: P,
    match_sep: M,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    M: Fn(&SpannedToken) -> bool,
    O: 'chunk,
{
    let mut list = List::default();
    let mut current = list.cursor_mut();

    let next = parser(lexer, alloc)?;
    current = current.alloc_insert_advance(alloc, next);

    while let Some(sep) = lexer.next_if(&match_sep) {
        if let Some(next) = parser(lexer, alloc).recover()? {
            current = current.alloc_insert_advance(alloc, next);
        } else {
            lexer.reset(sep);
            break;
        }
    }

    Ok(list)
}

pub(crate) fn parse_separated_list0<'chunk, 'src, P, M, O>(
    lexer: &mut PeekableLexer<'src, '_>,
    alloc: &'chunk ASTAllocator,
    parser: P,
    match_sep: M,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    M: Fn(&SpannedToken) -> bool,
    O: 'chunk,
{
    parse_separated_list1(lexer, alloc, parser, match_sep)
        .recover()
        .map(Option::unwrap_or_default)
}

pub fn parse_chunk<'src, 'strings, 'chunk>(
    input: &'src str,
    alloc: &'chunk ASTAllocator,
    strings: &'strings mut StringTable,
) -> Result<Block<'chunk>, ChunkParseError> {
    if input.is_empty() {
        Ok(Block::default())
    } else {
        final_parser!((input.as_bytes(), alloc, strings) => Block::parse)
            .map_err(ChunkParseError::from)
    }
}

#[derive(Debug, Default, Clone)]
pub struct StringTable {
    strings: IndexSet<LuaString>,
}

impl StringTable {
    pub fn get_ident(&self, ident: Ident) -> Option<&LuaString> {
        self.strings.get_index(ident.0)
    }

    pub fn get_string(&self, string: ConstantString) -> Option<&LuaString> {
        self.strings.get_index(string.0)
    }

    pub fn lookup_ident<'s>(&self, ident: impl Into<&'s BStr>) -> Option<Ident> {
        self.strings.get_index_of(ident.into()).map(Ident)
    }

    pub fn add_ident<'s>(&mut self, ident: impl Into<&'s BStr> + Copy) -> Ident {
        if let Some(id) = self.strings.get_index_of(ident.into()) {
            Ident(id)
        } else {
            Ident(self.strings.insert_full(ident.into().to_owned().into()).0)
        }
    }

    pub fn add_string(&mut self, string: BString) -> ConstantString {
        ConstantString(self.strings.insert_full(string.into()).0)
    }
}

#[derive(Debug)]
pub struct ASTAllocator(Bump);

impl ASTAllocator {
    pub fn allocated_bytes(&self) -> usize {
        self.0.allocated_bytes()
    }

    #[allow(clippy::mut_from_ref)] // I think bumpalo knows what it's doing
    pub fn alloc<T>(&self, val: T) -> &mut T {
        self.0.alloc(val)
    }
}

impl Default for ASTAllocator {
    fn default() -> Self {
        Self(Bump::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::Block,
        parse_chunk,
        ASTAllocator,
        StringTable,
    };

    #[test]
    pub fn parses_empty_chunk() -> anyhow::Result<()> {
        let src = "";

        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let result = parse_chunk(src, &alloc, &mut strings)?;

        assert_eq!(
            result,
            Block {
                statements: Default::default(),
                ret: None
            }
        );

        Ok(())
    }

    #[cfg(feature = "rendered-errors")]
    #[test]
    #[ignore = "just for interacting with error output"]
    fn dbg_rendered_error() {
        use ariadne::Source;

        use crate::{
            parse_chunk,
            ASTAllocator,
            StringTable,
        };

        let src = "if true then a ";
        let alloc = ASTAllocator::default();
        let mut strings = StringTable::default();
        let err = parse_chunk(src, &alloc, &mut strings);

        match err {
            Ok(_) => assert!(dbg!(err).is_err()),
            Err(e) => {
                let report = e.build_report();
                report.eprint(Source::from(src)).unwrap();
            }
        }
    }
}
