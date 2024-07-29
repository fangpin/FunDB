use std::cmp::Ordering;

/// Comparator trait, supporting types that can be nested (i.e., add additional functionality on
/// top of an inner comparator)
pub trait Cmp {
    /// Compare to byte strings, bytewise.
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering;

    /// Return the shortest byte string that compares "Greater" to the first argument and "Less" to
    /// the second one.
    fn find_shortest_sep(&self, from: &[u8], to: &[u8]) -> Vec<u8>;

    /// Return the shortest byte string that compares "Greater" to the argument.
    fn find_shortest_succ(&self, a: &[u8]) -> Vec<u8>;

    // A unique identifier for a comparator. A comparator wrapper (like InternalKeyCmp) may
    /// return the id of its inner comparator.
    fn id(&self) -> &'static str;
}

struct DefaultCmp;
impl Cmp for DefaultCmp{
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }

    fn id(&self) -> &'static str {
        "fundb.BytewiseCmp"
    }

    fn find_shortest_succ(&self, a: &[u8]) -> Vec<u8> {
        let mut result: Vec<_> = a.into_iter().copied().take_while(|c| *c == 0xff).collect();
        result.push(if result.len() == a.len() { 0xff } else { a[result.len()] + 1 });
        result
    }

    fn find_shortest_sep(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        vec![0xff]
    }
}