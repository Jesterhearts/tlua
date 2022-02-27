use crate::{
    errors::{
        ParseError,
        ParseErrorExt,
    },
    lexer::SpannedToken,
    list::List,
    ASTAllocator,
    PeekableLexer,
};

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
