use bstr::{
    BStr,
    BString,
};
use bumpalo::Bump;
use indexmap::IndexSet;
use logos::Lexer;
use nom::Offset;
use tlua_strings::LuaString;

pub mod block;
mod combinators;
pub mod errors;
pub mod expressions;
pub mod identifiers;
mod lexer;
pub mod list;
pub mod prefix_expression;
pub mod statement;

pub(crate) use combinators::*;
pub use errors::ChunkParseError;
pub(crate) use errors::{
    ParseError,
    ParseErrorExt,
    SyntaxError,
};

use crate::{
    block::Block,
    expressions::strings::ConstantString,
    identifiers::Ident,
    lexer::{
        SpannedToken,
        Token,
    },
};

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

    fn expecting_token_or(
        &mut self,
        token: Token,
        err: SyntaxError,
    ) -> Result<SpannedToken<'src>, ParseError> {
        self.next_if_eq(token)
            .ok_or_else(|| ParseError::recoverable_from_here(self, err))
    }

    fn expecting_token(&mut self, token: Token) -> Result<SpannedToken<'src>, ParseError> {
        self.expecting_token_or(token, SyntaxError::ExpectedToken(token))
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

macro_rules! final_parser {
    (($input:expr, $alloc:expr, $strings:expr) => $parser:expr) => {{
        let mut token_stream = $crate::SpannedTokenStream::new($input, $strings);
        let res = $parser(&mut token_stream, $alloc).and_then(|val| match token_stream.peek() {
            None => Ok(val),
            Some(token) => Err($crate::ParseError {
                error: $crate::errors::SyntaxError::ExpectedEOF(token.token),
                location: token.span,
                recoverable: false,
            }),
        });

        res
    }};
}

pub(crate) use final_parser;

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
