use bstr::{
    BStr,
    BString,
};
use bumpalo::Bump;
use indexmap::IndexSet;
use logos::Lexer;
use nom::Offset;
use nom_supreme::error::ErrorTree;
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
#[error("Errors parsing chunk: {errors:#}")]
pub struct ChunkParseError {
    pub errors: ErrorTree<SourceSpan>,
}

#[cfg(feature = "rendered-errors")]
impl ChunkParseError {
    pub fn build_report(&self) -> ariadne::Report<std::ops::Range<usize>> {
        use ariadne::{
            Label,
            Report,
            ReportKind,
        };
        use indexmap::indexset;

        let mut labels = indexset! {};
        Self::build_tree(&mut labels, &self.errors);

        let mut builder = Report::build(
            ReportKind::Error,
            (),
            labels.first().map(|(span, _)| span.start).unwrap_or(0),
        )
        .with_message("Failed to parse LUA");

        for (range, label) in labels {
            builder = builder.with_label(Label::new(range.start..range.end).with_message(label));
        }

        builder.finish()
    }

    fn build_tree(
        tree: &mut indexmap::IndexSet<(SourceSpan, String)>,
        err: &ErrorTree<SourceSpan>,
    ) {
        use nom_supreme::error::BaseErrorKind;

        match err {
            ErrorTree::Base { location, kind } => {
                let label = match kind {
                    BaseErrorKind::External(e) => e.to_string(),
                    k => k.to_string(),
                };
                tree.insert((*location, label));
            }
            ErrorTree::Stack { base, contexts } => {
                tree.extend(
                    contexts
                        .iter()
                        .rev()
                        .map(|(span, context)| (*span, context.to_string())),
                );
                Self::build_tree(tree, base);
            }
            ErrorTree::Alt(cases) => {
                for case in cases {
                    Self::build_tree(tree, case);
                }
            }
        }
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
    location: Option<SourceSpan>,
    recoverable: bool,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(location) = self.location {
            f.write_fmt(format_args!("Error: {} at {}", self.error, location))
        } else {
            f.write_fmt(format_args!("Error: {} at eof", self.error))
        }
    }
}

impl ParseError {
    pub(crate) fn recoverable_from_here(lexer: &mut PeekableLexer, err: SyntaxError) -> Self {
        Self {
            error: err,
            location: lexer.peek().map(SpannedToken::into_span),
            recoverable: true,
        }
    }

    pub(crate) fn unrecoverable_from_here(lexer: &mut PeekableLexer, err: SyntaxError) -> Self {
        Self {
            error: err,
            location: lexer.peek().map(SpannedToken::into_span),
            recoverable: false,
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
            strings,
            peeked: None,
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
        self.lexer.bump(self.src.offset(token.src));
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
    #[error("Expected end of file, found: {0:?}")]
    ExpectedEOF(Token),
    #[error("Expected {0:?}")]
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

    fn ok_or_else<F: FnOnce() -> Self::Error>(self, err: F) -> Result<Self::Data, Self::Error> {
        self.recover()?.ok_or_else(err)
    }
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
            Err(e) => {
                if e.recoverable {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    #[inline]
    fn recover_with<F: FnOnce() -> Result<Self::Data, Self::Error>>(
        self,
        recover: F,
    ) -> Result<Self::Data, Self::Error> {
        match self {
            Ok(data) => Ok(data),
            Err(e) => {
                if e.recoverable {
                    recover()
                } else {
                    Err(e)
                }
            }
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
                location: Some(token.span),
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

pub(crate) fn parse_list0_split_tail<'chunk, 'src, P, O>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<(List<'chunk, O>, Option<O>), ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    O: 'chunk,
{
    parse_list1_split_tail(lexer, alloc, parser)
        .recover()
        .map(|maybe_list| match maybe_list {
            Some((list, tail)) => (list, Some(tail)),
            None => (Default::default(), None),
        })
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

pub(crate) fn parse_list0<'chunk, 'src, P, O>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<O, ParseError>,
    O: 'chunk,
{
    parse_list1(lexer, alloc, parser)
        .recover()
        .map(Option::unwrap_or_default)
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
    let _block = final_parser!((input.as_bytes(), alloc, strings) => Block::parse);
    todo!()
}

#[derive(Debug, Default, Clone)]
pub struct StringTable {
    idents: IndexSet<LuaString>,
    strings: IndexSet<LuaString>,
}

impl StringTable {
    pub fn get_ident(&self, ident: Ident) -> Option<&LuaString> {
        self.idents.get_index(ident.0)
    }

    pub fn get_string(&self, string: ConstantString) -> Option<&LuaString> {
        self.strings.get_index(string.0)
    }

    pub fn lookup_ident<'s>(&self, ident: impl Into<&'s BStr>) -> Option<Ident> {
        self.idents.get_index_of(ident.into()).map(Ident)
    }

    pub fn add_ident<'s>(&mut self, ident: impl Into<&'s BStr> + Copy) -> Ident {
        if let Some(id) = self.idents.get_index_of(ident.into()) {
            Ident(id)
        } else {
            Ident(self.idents.insert_full(ident.into().to_owned().into()).0)
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

        let src = "if true then 10";
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
