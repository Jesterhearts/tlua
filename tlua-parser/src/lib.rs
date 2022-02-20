use bumpalo::Bump;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    combinator::value,
    sequence::{
        delimited,
        preceded,
    },
    IResult,
};
use nom_supreme::{
    error::ErrorTree,
    ParserExt,
};
use thiserror::Error;

pub mod block;
pub mod comments;
pub mod expressions;
pub mod identifiers;
pub mod list;
pub mod prefix_expression;
pub mod statement;
pub mod string;

use self::comments::parse_comment;
use crate::block::Block;

#[derive(Debug, Error)]
#[error("Errors parsing chunk: {errors:#}")]
pub struct ChunkParseError {
    pub errors: ErrorTree<ErrorSpan>,
}

#[derive(Debug)]
pub struct ErrorSpan {
    start: usize,
    end: usize,
}

impl std::fmt::Display for ErrorSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "bytes: {start}..{end}",
            start = self.start,
            end = self.end
        ))
    }
}

impl From<Span<'_>> for ErrorSpan {
    fn from(s: Span<'_>) -> Self {
        Self {
            start: s.location_offset(),
            end: s.location_offset() + s.len(),
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum SyntaxError {
    #[error("decimal escape too large")]
    DecimalEscapeTooLarge,
    #[error("UTF-8 value too large")]
    Utf8ValueTooLarge,
    #[error("Malformed number")]
    MalformedNumber,
    #[error("Integer constant can't fit in integer")]
    IntegerConstantTooLarge,
    #[error("Keyword cannot be used as identifier")]
    KeywordAsIdent,
    #[error("Expected a variable declaration")]
    ExpectedVariable,
    #[error("Expected a function call or a variable, encountered a parenthesized expression")]
    ExpectedVarOrCall,
    #[error("Invalid attribute - expected <const> or <close>")]
    InvalidAttribute,
}

// Newline, carriage return, tab, vertical tab, form feed, space
pub const LUA_WHITESPACE: &[u8] = b"\n\r\t\x0B\x0C ";

pub type Span<'a> = nom_locate::LocatedSpan<&'a [u8]>;
pub type InternalLuaParseError<'a> = nom_supreme::error::ErrorTree<Span<'a>>;
pub type ParseResult<'a, T> = IResult<Span<'a>, T, InternalLuaParseError<'a>>;

#[macro_export]
macro_rules! final_parser {
    ($input:expr => $parser:expr) => {
        ::nom_supreme::final_parser::final_parser($parser)($input)
            .map_err(|e: crate::InternalLuaParseError| e.map_locations(crate::ErrorSpan::from))
            .map_err(|errors| $crate::ChunkParseError { errors })
    };
}

pub fn lua_whitespace0(mut input: Span) -> ParseResult<()> {
    loop {
        input = match alt((
            value((), take_while1(|c| LUA_WHITESPACE.contains(&c))).context("whitespace"),
            parse_comment,
        ))(input)
        {
            Err(_) => return Ok((input, ())),
            Ok((input, _)) => input,
        }
    }
}

pub fn lua_whitespace1(input: Span) -> ParseResult<()> {
    preceded(
        alt((
            value((), take_while1(|c| LUA_WHITESPACE.contains(&c))),
            parse_comment,
        )),
        lua_whitespace0,
    )(input)
}

pub fn parse_chunk<'src, 'chunk>(
    input: &'src str,
    alloc: &'chunk ASTAllocator,
) -> Result<Block<'chunk>, ChunkParseError> {
    final_parser!(
    Span::new(input.as_bytes()) =>
                delimited(
                lua_whitespace0,
                Block::parser(alloc),
                lua_whitespace0,
            ))
}

pub fn is_keyword(span: Span) -> bool {
    matches!(
        *span,
        b"and"
            | b"break"
            | b"do"
            | b"else"
            | b"elseif"
            | b"end"
            | b"false"
            | b"for"
            | b"function"
            | b"goto"
            | b"if"
            | b"in"
            | b"local"
            | b"nil"
            | b"not"
            | b"or"
            | b"repeat"
            | b"return"
            | b"then"
            | b"true"
            | b"until"
            | b"while"
    )
}

#[cfg(test)]
mod tests {

    use crate::{
        parse_chunk,
        ASTAllocator,
    };

    #[test]
    fn foo() {
        let src = "abc";
        let alloc = ASTAllocator::default();
        let err = dbg!(parse_chunk(src, &alloc));

        if !matches!(err, Err(_)) {
            assert!(err.is_ok());
        }
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
