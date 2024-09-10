/// An extension of the standard `Iterator` trait that supports some methods necessary for LevelDB.
/// This works because the iterators used are stateful and keep the last returned element.
///
/// Note: Implementing types are expected to hold `!valid()` before the first call to `advance()`.
///
/// test_util::test_iterator_properties() verifies that all properties hold.
pub trait LdbIterator {
    /// Advances the position of the iterator by one element (which can be retrieved using
    /// current(). If no more elements are available, advance() returns false, and the iterator
    /// becomes invalid (i.e. as if reset() had been called).
    fn advance(&mut self) -> bool;
    /// Return the current item (i.e. the item most recently returned by `next()`).
    fn current(&self, key: &mut Vec<u8>, val: &mut Vec<u8>) -> bool;
    /// Seek the iterator to `key` or the next bigger key. If the seek is invalid (past last
    /// element, or before first element), the iterator is `reset()` and not valid.
    fn seek(&mut self, key: &[u8]);
    /// Resets the iterator to be `!valid()`, i.e. positioned before the first element.
    fn reset(&mut self);
    /// Returns true if the iterator is not positioned before the first or after the last element,
    /// i.e. if `current()` would succeed.
    fn valid(&self) -> bool;
    /// Go to the previous item; if the iterator is moved beyond the first element, `prev()`
    /// returns false and it will be `!valid()`. This is inefficient for most iterator
    /// implementations.
    fn prev(&mut self) -> bool;

    // default implementations.

    /// next is like Iterator::next(). It's implemented here because Rust disallows implementing a
    /// foreign trait for any type, thus we can't do `impl<T: LdbIterator> Iterator<Item=Vec<u8>>
    /// for T {}`.
    fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        if !self.advance() {
            return None;
        }
        let (mut key, mut val) = (vec![], vec![]);
        if self.current(&mut key, &mut val) {
            Some((key, val))
        } else {
            None
        }
    }

    /// seek_to_first seeks to the first element.
    fn seek_to_first(&mut self) {
        self.reset();
        self.advance();
    }
}