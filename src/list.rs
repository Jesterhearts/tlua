use std::fmt::Debug;

use crate::ast::ASTAllocator;

#[derive(Debug)]
pub struct ListNode<'list, T> {
    value: T,
    next: Option<&'list mut ListNode<'list, T>>,
}

impl<'list, T> ListNode<'list, T> {
    pub(crate) fn value(&self) -> &T {
        &self.value
    }

    pub(crate) fn next<'s>(&'s self) -> Option<&'s ListNode<'list, T>> {
        self.next.as_deref()
    }
}

impl<'list, T> ListNode<'list, T> {
    fn set_next<'s>(
        &'s mut self,
        next: &'list mut ListNode<'list, T>,
    ) -> &'s mut ListNode<'list, T> {
        *self.next.insert(next)
    }
}

impl<T> ListNode<'_, T> {
    pub(crate) fn new(value: T) -> Self {
        Self { value, next: None }
    }
}

pub(crate) struct CursorMut<'c, 'list, T> {
    len: &'c mut usize,
    current: &'c mut Option<&'list mut ListNode<'list, T>>,
}

impl<'c, 'list, T> CursorMut<'c, 'list, T> {
    /// Advance cursor to the end of the list.
    pub(crate) fn advance_to_end(self) -> Self {
        if self.current.is_none() {
            return self;
        }
        let Self { len, mut current } = self;

        while current.as_deref_mut().unwrap().next.is_some() {
            current = &mut current.as_deref_mut().unwrap().next;
        }

        Self { len, current }
    }

    /// Allocate a list node, insert it after the current node and advance to
    /// it.
    pub(crate) fn alloc_insert_advance(self, alloc: &'list ASTAllocator, data: T) -> Self {
        self.insert_advance(alloc.alloc(ListNode::new(data)))
    }

    /// Insert next after the current node and advance to it.
    pub(crate) fn insert_advance(self, next: &'list mut ListNode<'list, T>) -> Self {
        let Self { len, current } = self;
        *len += 1;

        if let Some(current) = current {
            if let Some(current_next) = current.next.take() {
                next.set_next(current_next);
            }
            current.set_next(next);

            Self {
                len,
                current: &mut current.next,
            }
        } else {
            *current = Some(next);
            Self { len, current }
        }
    }
}

pub(crate) struct List<'list, T> {
    len: usize,
    head: Option<&'list mut ListNode<'list, T>>,
}

impl<'list, T> List<'list, T> {
    pub(crate) fn cursor_mut<'c>(&'c mut self) -> CursorMut<'c, 'list, T> {
        CursorMut {
            len: &mut self.len,
            current: &mut self.head,
        }
    }

    pub(crate) fn new(head: &'list mut ListNode<'list, T>) -> Self {
        Self {
            len: 1,
            head: Some(head),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.head.is_some()
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn head(&self) -> Option<&'_ ListNode<'list, T>> {
        self.head.as_deref()
    }

    pub(crate) fn from_slice(nodes: &'list mut [ListNode<'list, T>]) -> Self {
        let mut list = Self::default();
        let mut current = list.cursor_mut();
        for node in nodes {
            current = current.insert_advance(node);
        }

        list
    }

    pub(crate) fn iter<'a>(&'a self) -> Iter<'a, 'list, T> {
        Iter {
            remain: self.len,
            current: self.head(),
        }
    }
}

impl<'o, 'l, L, O> PartialEq<List<'o, O>> for List<'l, L>
where
    L: PartialEq<O>,
    O: PartialEq<L>,
{
    fn eq(&self, other: &List<'o, O>) -> bool {
        *self == other.iter()
    }
}

impl<'i, 'l, L, O, I> PartialEq<I> for List<'l, L>
where
    I: Iterator<Item = &'i O> + Clone,
    L: PartialEq<O>,
    O: PartialEq<L> + 'i,
{
    fn eq(&self, other: &I) -> bool {
        let mut this = self.iter();
        let mut other = (*other).clone();
        loop {
            match (this.next(), other.next()) {
                (None, None) => return true,
                (Some(t), Some(o)) => {
                    if t != o {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    }
}

impl<'l, L, O> PartialEq<[O]> for List<'l, L>
where
    L: PartialEq<O>,
    O: PartialEq<L>,
{
    fn eq(&self, other: &[O]) -> bool {
        *self == other.iter()
    }
}

impl<T> Debug for List<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Default for List<'_, T> {
    fn default() -> Self {
        Self { len: 0, head: None }
    }
}

#[derive(Debug)]
pub(crate) struct Iter<'i, 'list, T> {
    remain: usize,
    current: Option<&'i ListNode<'list, T>>,
}

impl<T> Copy for Iter<'_, '_, T> {}

impl<T> Clone for Iter<'_, '_, T> {
    fn clone(&self) -> Self {
        Self {
            remain: self.remain,
            current: self.current,
        }
    }
}

impl<'i, 'list, T> Iterator for Iter<'i, 'list, T> {
    type Item = &'i T;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remain, Some(self.remain))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            self.remain -= 1;
            let value = &current.value;
            self.current = current.next.as_deref();
            Some(value)
        } else {
            debug_assert!(self.remain == 0);
            None
        }
    }
}

impl<'i, 'list, T> ExactSizeIterator for Iter<'i, 'list, T> {
    fn len(&self) -> usize {
        self.remain
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{
        List,
        ListNode,
    };

    #[test]
    fn appends() {
        let mut node1 = ListNode::new(1);
        let mut node2 = ListNode::new(2);
        let mut node3 = ListNode::new(3);

        let mut list = List::default();
        let cursor = list.cursor_mut();

        cursor
            .insert_advance(&mut node1)
            .insert_advance(&mut node2)
            .insert_advance(&mut node3);

        assert_eq!(list.len(), 3);
        assert_eq!(list, [1, 2, 3][..]);
    }

    #[test]
    fn inserts_middle() {
        let mut node1 = ListNode::new(1);
        let mut node2 = ListNode::new(2);
        let mut node3 = ListNode::new(3);

        let mut list = List::default();
        list.cursor_mut()
            .insert_advance(&mut node1)
            .insert_advance(&mut node2);

        let cursor = list.cursor_mut();
        cursor.insert_advance(&mut node3);

        assert_eq!(list.len(), 3);
        assert_eq!(list, [1, 3, 2][..]);
    }

    #[test]
    fn iterates() {
        assert_eq!(
            List::from_slice(&mut [ListNode::new(1), ListNode::new(2)]),
            [1, 2][..]
        );
    }
}
