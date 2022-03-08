use crate::{
    errors::{
        ParseError,
        SyntaxError,
    },
    lexer::SpannedToken,
    list::List,
    ASTAllocator,
    PeekableLexer,
};

pub(crate) fn parse_list_with_head_split_tail<'chunk, 'src, P, O>(
    head: O,
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<(List<'chunk, O>, O), ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    O: 'chunk,
{
    let mut list = List::default();
    let mut current = list.cursor_mut();

    let mut prev = head;
    loop {
        if let Some(next) = parser(lexer, alloc)? {
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
) -> Result<Option<(List<'chunk, O>, O)>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    O: 'chunk,
{
    parser(lexer, alloc).and_then(|head| {
        head.map_or(Ok(None), |head| {
            parse_list_with_head_split_tail(head, lexer, alloc, parser).map(Some)
        })
    })
}

pub(crate) fn parse_list1_split_tail_or<'chunk, 'src, P, O>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
    err: SyntaxError,
) -> Result<(List<'chunk, O>, O), ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    O: 'chunk,
{
    parse_list0_split_tail(lexer, alloc, parser)
        .and_then(|data| data.map_or_else(|| Err(ParseError::from_here(lexer, err)), Ok))
}

pub(crate) fn parse_list_with_head<'chunk, 'src, P, O>(
    head: O,
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    O: 'chunk,
{
    let mut list = List::default();
    let mut current = list.cursor_mut();
    current = current.alloc_insert_advance(alloc, head);

    while let Some(next) = parser(lexer, alloc)? {
        current = current.alloc_insert_advance(alloc, next);
    }

    Ok(list)
}

pub(crate) fn parse_list0<'chunk, 'src, P, O>(
    lexer: &mut PeekableLexer,
    alloc: &'chunk ASTAllocator,
    parser: P,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    O: 'chunk,
{
    parser(lexer, alloc).and_then(|head| {
        head.map_or_else(
            || Ok(Default::default()),
            |head| parse_list_with_head(head, lexer, alloc, parser),
        )
    })
}

pub(crate) fn parse_separated_list_with_head<'chunk, 'src, P, M, O>(
    head: O,
    lexer: &mut PeekableLexer<'src, '_>,
    alloc: &'chunk ASTAllocator,
    parser: P,
    match_sep: M,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    M: Fn(&SpannedToken) -> bool,
    O: 'chunk,
{
    let mut list = List::default();
    let mut current = list.cursor_mut();

    current = current.alloc_insert_advance(alloc, head);

    while let Some(sep) = lexer.next_if(&match_sep) {
        if let Some(next) = parser(lexer, alloc)? {
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
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    M: Fn(&SpannedToken) -> bool,
    O: 'chunk,
{
    parser(lexer, alloc).and_then(|head| {
        head.map_or_else(
            || Ok(Default::default()),
            |head| parse_separated_list_with_head(head, lexer, alloc, parser, match_sep),
        )
    })
}

pub(crate) fn parse_separated_list1_or<'chunk, 'src, P, M, O>(
    lexer: &mut PeekableLexer<'src, '_>,
    alloc: &'chunk ASTAllocator,
    parser: P,
    match_sep: M,
    err: SyntaxError,
) -> Result<List<'chunk, O>, ParseError>
where
    P: Fn(&mut PeekableLexer, &'chunk ASTAllocator) -> Result<Option<O>, ParseError>,
    M: Fn(&SpannedToken) -> bool,
    O: 'chunk,
{
    parse_separated_list0(lexer, alloc, parser, match_sep).and_then(|data| {
        if data.is_empty() {
            Err(ParseError::from_here(lexer, err))
        } else {
            Ok(data)
        }
    })
}
