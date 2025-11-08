/// A reference-based immutable linked list that doesn't own its data
#[derive(Debug, Clone, Copy)]
pub enum RefList<'a, T> {
    Nil,
    Cons(T, &'a RefList<'a, T>),
}

impl<'a, T> RefList<'a, T> {
    /// Create a new list by prepending an element to this list
    pub fn prepend<'b>(&'b self, item: T) -> RefList<'b, T> 
    where 
        'a: 'b 
    {
        RefList::Cons(item, self)
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, RefList::Nil)
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        match self {
            RefList::Nil => 0,
            RefList::Cons(_, tail) => 1 + tail.len(),
        }
    }

    /// Get an iterator over the list
    pub fn iter(&'a self) -> RefListIter<'a, T> {
        RefListIter { current: self }
    }

    /// Get the nth head element (0-based) from the list
    pub fn headn(&self, n: usize) -> Option<&T> {
        let mut idx = 0;
        let mut current = self;
        loop {
            match current {
                RefList::Nil => return None,
                RefList::Cons(item, tail) => {
                    if idx == n {
                        return Some(item);
                    }
                    idx += 1;
                    current = tail;
                }
            }
        }
    }
}

impl<'a, T> IntoIterator for &RefList<'a, T> {
    type Item = &'a T;
    type IntoIter = RefListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator for RefList
pub struct RefListIter<'a, T> {
    current: &'a RefList<'a, T>,
}

impl<'a, T> Iterator for RefListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            RefList::Nil => None,
            RefList::Cons(item, tail) => {
                self.current = tail;
                Some(item)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflist_basic() {
        let empty = RefList::Nil;
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let list1 = empty.prepend(1);
        assert!(!list1.is_empty());
        assert_eq!(list1.len(), 1);

        let list2 = list1.prepend(2);
        assert_eq!(list2.len(), 2);

        let items: Vec<_> = list2.iter().copied().collect();
        assert_eq!(items, vec![2, 1]);
    }
}