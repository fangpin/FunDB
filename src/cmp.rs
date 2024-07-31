use std::{cmp::Ordering, result};

/// Comparator trait, supporting types that can be nested (i.e., add additional functionality on
/// top of an inner comparator)
pub trait Cmp {
    /// Compare to byte strings, bytewise.
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering;

    /// Return the shortest byte string that compares "Greater" to the first argument and "Less" to
    /// the second one.
    fn find_shortest_sep(&self, from: Vec<u8>, to: &[u8]) -> Vec<u8>;

    /// Return the shortest byte string that compares "Greater" to the argument.
    fn find_short_succ(&self, a: Vec<u8>) -> Vec<u8>;

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

    fn find_short_succ(&self, a: Vec<u8>) -> Vec<u8> {
        let mut result: Vec<_> = a.iter().copied().take_while(|c| *c == 0xff).collect();
        result.push(if result.len() == a.len() { 0xff } else { a[result.len()] + 1 });
        result
    }

    fn find_shortest_sep(&self, from: Vec<u8>, to: &[u8]) -> Vec<u8> {
        let min_len = if from.len() < to.len() { from.len() } else { to.len() };
        let mut ret: Vec<_> = from.iter().zip(to.iter()).take_while(|(a, b)| { **a == **b}).map(|(a, _)| *a).collect();
        if ret.len() <= min_len && from[ret.len()] < 0xff && from[ret.len()] + 1 < to[ret.len()] { 
            ret.push(from[ret.len()] + 1)
        }
        ret 
    }
}