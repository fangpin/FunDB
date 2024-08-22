/// An extension of the standard `Iterator` trait that supports some methods necessary for DB.
/// This works because the iterators used are stateful and keep the last returned element.
pub trait DBIter {
    /// return the next value and advance the iterator
    fn next(&mut self) -> Option<(&[u8], &[u8])>;

    /// return the next value without advancing the iterator
    fn peek(&self) -> Option<(&[u8], &[u8])>;

    /// seek the iterator to the 'key' or greater 'key'
    fn seek(&mut self, key: &[u8]);

    /// reset the iterator to the beginning
    fn reset(&mut self);

    /// go to the previous position
    fn prev(&mut self);
}