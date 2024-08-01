use std::{cmp::Ordering, rc::Rc, result};

use crate::{ktypes, types};

/// Comparator trait, supporting types that can be nested (i.e., add additional functionality on
/// top of an inner comparator)
pub trait Cmp {
    /// Compare to byte strings, bytewise.
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering;

    /// Return the shortest byte string that compares "Greater" to the first argument and "Less" to
    /// the second one.
    fn find_shortest_sep(&self, from: &[u8], to: &[u8]) -> Vec<u8>;

    /// Return the shortest byte string that compares "Greater" to the argument.
    fn find_short_succ(&self, a: &[u8]) -> Vec<u8>;

    // A unique identifier for a comparator. A comparator wrapper (like InternalKeyCmp) may
    /// return the id of its inner comparator.
    fn id(&self) -> &'static str;
}

/// default comparator
struct DefaultCmp;
impl Cmp for DefaultCmp{
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering {
        a.cmp(b)
    }

    fn id(&self) -> &'static str {
        "fundb.BytewiseCmp"
    }

    fn find_short_succ(&self, a: &[u8]) -> Vec<u8> {
        let mut result: Vec<_> = a.iter().copied().take_while(|c| *c == 0xff).collect();
        result.push(if result.len() == a.len() { 0xff } else { a[result.len()] + 1 });
        result
    }

    fn find_shortest_sep(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        let min_len = if from.len() < to.len() { from.len() } else { to.len() };
        let mut ret: Vec<_> = from.iter().zip(to.iter()).take_while(|(a, b)| { **a == **b}).map(|(a, _)| *a).collect();
        if ret.len() <= min_len && from[ret.len()] < 0xff && from[ret.len()] + 1 < to[ret.len()] { 
            ret.push(from[ret.len()] + 1)
        }
        ret 
    }
}

struct InternalKeyCmp(pub Rc<dyn Cmp>);
impl Cmp for InternalKeyCmp {
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering {
        ktypes::cmp_internal_key(self.0.as_ref(), a, b)
    }

    fn id(&self) -> &'static str {
        self.0.id()
    }

    fn find_shortest_sep(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        if a == b {
            return a.to_vec();
        }

        let (key_a, seq_a, tpa) = ktypes::parse_internal_key(a);
        let (key_b, _, _) = ktypes::parse_internal_key(b);

        let sep: Vec<u8> = self.0.find_shortest_sep(key_a, key_b);

        if sep.len() < key_a.len() && self.0.cmp(key_a, &sep) == Ordering::Less {
            return ktypes::LookupKey::new(&sep, types::MAX_SEQUENCE_NUMBER, tpa)
                .internal_key()
                .to_vec();
        }
        ktypes::LookupKey::new(&sep, seq_a, tpa).internal_key().to_vec()
    }

    fn find_short_succ(&self, a: &[u8]) -> Vec<u8> {
        let (key, seq, typ) = ktypes::parse_internal_key(a);
        let succ: Vec<u8> = self.0.find_short_succ(key);
        ktypes::LookupKey::new(&succ, seq, typ).internal_key().to_vec()
    }
}

/// mem key comparator
struct MemCmp(pub Rc<dyn Cmp>);
impl Cmp for MemCmp {
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering {
        ktypes::cmp_mem_key(self.0.as_ref(), a, b)
    }

    fn find_short_succ(&self, a: &[u8]) -> Vec<u8> {
        panic!("find_short_succ should not be used for MemCmp");
    }

    fn find_shortest_sep(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        panic!("find_shortest_sep should not be used for MemCmp");
    }

    fn id(&self) -> &'static str {
        self.0.id()
    }
}