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
pub struct DefaultCmp;
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
        if from == to {
            return from.to_vec();
        }
        let min = if from.len() < to.len() { from.len() } else { to.len() };
        let mut diff_at = 0;

        while diff_at < min && from[diff_at] == to[diff_at] {
            diff_at += 1;
        }

        // First, try to find a short separator. If that fails, try a backup mechanism below.
        while diff_at < min {
            let diff = from[diff_at];
            if diff < 0xff && diff + 1 < to[diff_at] {
                let mut sep = Vec::from(&from[0..diff_at + 1]);
                sep[diff_at] += 1;
                assert!(self.cmp(&sep, to) == Ordering::Less);
                return sep;
            }

            diff_at += 1;
        }

        let mut sep = Vec::with_capacity(from.len() + 1);
        sep.extend_from_slice(from);
        // Try increasing a and check if it's still smaller than b. First find the last byte
        // smaller than 0xff, and then increment that byte. Only if the separator is lesser than b,
        // return it.
        let mut i = from.len() - 1;
        while i > 0 && sep[i] == 0xff {
            i -= 1;
        }
        if sep[i] < 0xff {
            sep[i] += 1;
            if self.cmp(&sep, to) == Ordering::Less {
                return sep;
            } else {
                sep[i] -= 1;
            }
        }

        // Backup case: either `a` is full of 0xff, or all different places are less than 2
        // characters apart.
        // The result is not necessarily short, but a good separator: e.g., "abc" vs "abd" ->
        // "abc\0", which is greater than abc and lesser than abd.
        // Append a 0 byte; by making it longer than a, it will compare greater to it.
        sep.extend_from_slice(&[0]);
        sep
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
pub struct MemKeyCmp(pub Rc<Box<dyn Cmp>>);
impl Cmp for MemKeyCmp {
    fn cmp(&self, a: &[u8], b: &[u8]) -> Ordering {
        ktypes::cmp_mem_key(self.0.as_ref().as_ref(), a, b)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ktypes::LookupKey;

    #[test]
    fn test_cmp_defaultcmp_shortest_sep() {
        assert_eq!(
            DefaultCmp.find_shortest_sep("abcd".as_bytes(), "abcf".as_bytes()),
            "abce".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("abc".as_bytes(), "acd".as_bytes()),
            "abd".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("abcdefghi".as_bytes(), "abcffghi".as_bytes()),
            "abce".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("a".as_bytes(), "a".as_bytes()),
            "a".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("a".as_bytes(), "b".as_bytes()),
            "a\0".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("abc".as_bytes(), "zzz".as_bytes()),
            "b".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("yyy".as_bytes(), "z".as_bytes()),
            "yyz".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_shortest_sep("".as_bytes(), "".as_bytes()),
            "".as_bytes()
        );
    }

    #[test]
    fn test_cmp_defaultcmp_short_succ() {
        assert_eq!(
            DefaultCmp.find_short_succ("abcd".as_bytes()),
            "b".as_bytes()
        );
        assert_eq!(
            DefaultCmp.find_short_succ("zzzz".as_bytes()),
            "{".as_bytes()
        );
        assert_eq!(DefaultCmp.find_short_succ(&[]), &[0xff]);
        assert_eq!(
            DefaultCmp.find_short_succ(&[0xff, 0xff, 0xff]),
            &[0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn test_cmp_internalkeycmp_shortest_sep() {
        let cmp = InternalKeyCmp(Rc::new(DefaultCmp));
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("abcd".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("abcf".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("abce".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key()
        );
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("abcd".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("abce".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("abcd\0".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key()
        );
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("abc".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("zzz".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("b".as_bytes(), types::MAX_SEQUENCE_NUMBER, ktypes::ValueType::TypeValue).internal_key()
        );
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("abc".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("acd".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("abd".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key()
        );
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("abc".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("abe".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("abd".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key()
        );
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key()
        );
        assert_eq!(
            cmp.find_shortest_sep(
                LookupKey::new("abc".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key(),
                LookupKey::new("abc".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
            ),
            LookupKey::new("abc".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key()
        );
    }

    #[test]
    fn test_cmp_internalkeycmp() {
        let cmp = InternalKeyCmp(Rc::new(DefaultCmp));
        // a < b < c
        let a = LookupKey::new("abc".as_bytes(), 2, ktypes::ValueType::TypeValue).internal_key().to_vec();
        let b = LookupKey::new("abc".as_bytes(), 1, ktypes::ValueType::TypeValue).internal_key().to_vec();
        let c = LookupKey::new("abd".as_bytes(), 3, ktypes::ValueType::TypeValue).internal_key().to_vec();
        let d = "xyy".as_bytes();
        let e = "xyz".as_bytes();

        assert_eq!(Ordering::Less, cmp.cmp(&a, &b));
        assert_eq!(Ordering::Equal, cmp.cmp(&a, &a));
        assert_eq!(Ordering::Greater, cmp.cmp(&b, &a));
        assert_eq!(Ordering::Less, cmp.cmp(&a, &c));
    }

    #[test]
    #[should_panic]
    fn test_cmp_memtablekeycmp_panics() {
        let cmp = MemKeyCmp(Rc::new(Box::new(DefaultCmp)));
        cmp.cmp(&[1, 2, 3], &[4, 5, 6]);
    }
}