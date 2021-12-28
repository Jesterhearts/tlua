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

use crate::ast::{
    block::Block,
    ASTAllocator,
};

pub mod block;
pub mod comments;
pub mod expressions;
pub mod identifiers;
pub mod prefix_expression;
pub mod statement;
pub mod string;

use self::comments::parse_comment;

#[derive(Debug, Error)]
#[error("Errors parsing chunk: {errors:#}")]
pub struct ChunkParseError {
    pub errors: ErrorTree<LuaParseError>,
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
    #[error("Invalid attribute - expected <const> or <close>, but got: {attrib:?}")]
    InvalidAttribute { attrib: String },
    #[error("No label matching {target:?}")]
    MissingGotoLabel { target: String },
}

// Newline, carriage return, tab, vertical tab, form feed, space
pub const LUA_WHITESPACE: &[u8] = b"\n\r\t\x0B\x0C ";

pub type Span<'a> = nom_locate::LocatedSpan<&'a [u8]>;
pub type InternalLuaParseError<'a> = nom_supreme::error::ErrorTree<Span<'a>>;
pub type ParseResult<'a, T> = IResult<Span<'a>, T, InternalLuaParseError<'a>>;

#[derive(Debug, Error, Clone, PartialEq)]
pub struct LuaParseError {
    line: u32,
    column: usize,
    text: String,
}

impl<'a> From<Span<'a>> for LuaParseError {
    fn from(err: Span<'a>) -> Self {
        Self {
            line: err.location_line(),
            column: err.get_utf8_column(),
            text: String::from_utf8_lossy(err.get_line_beginning()).to_string(),
        }
    }
}

impl std::fmt::Display for LuaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "line: {}, column: {} \n\t {}",
            self.line, self.column, self.text
        ))
    }
}

#[macro_export]
macro_rules! final_parser {
    ($input:expr => $parser:expr) => {
        ::nom_supreme::final_parser::final_parser($parser)($input)
            .map_err(|e: crate::parsing::InternalLuaParseError| {
                e.map_locations(crate::parsing::LuaParseError::from)
            })
            .map_err(|errors| $crate::parsing::ChunkParseError { errors })
    };
}

pub trait Parse<'chunk>: Sized {
    fn parse<'src>(input: Span<'src>, alloc: &'chunk ASTAllocator) -> ParseResult<'src, Self>;
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
                |input| Block::parse(input, alloc),
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
