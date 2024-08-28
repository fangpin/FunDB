use std::{cell::RefCell, cmp::Ordering, iter::{self, Skip}, mem::{self, size_of}, ptr::{self, null}, rc::Rc};
use rand::{rngs::StdRng, RngCore, SeedableRng};

use crate::{cmp::Cmp, iterator::DBIter};

const MAX_HEIGHT: usize = 12;
const BRANCHING_FACTOR: u32 = 4;

/// the internal node for inner skip list
struct Node {
    key: Vec<u8>,
    value: Vec<u8>,
    next: Option<Box<Node>>,
    skips: Vec<Option<*mut Node>>,
}

struct InnerSkipList {
    head: Box<Node>,
    len: usize,
    // approximate memory usage
    approx_mem: usize,
    cmp: Rc<Box<dyn Cmp>>,
    rand: StdRng,
}

impl Drop for InnerSkipList {
    fn drop(&mut self) {
        let mut cur = self.head.next.take();
        while let Some(mut node) = cur {
            cur = node.next.take();
        }
    }
}

impl InnerSkipList {
    fn random_height(&mut self) -> usize {
        let mut height = 1;
        while height < MAX_HEIGHT && self.rand.next_u32() % BRANCHING_FACTOR == 0 { 
            height += 1;
        }
        height
    }

    fn get_greater_or_equal<'a>(&'a self, key: &[u8]) -> Option<&'a Node> {
        let mut cur = self.head.as_ref() as *const Node;
        let mut level = self.head.skips.len() - 1;
        loop {
            unsafe {
                if let Some(next) = (*cur).skips[level] {
                    match self.cmp.cmp((*next).key.as_slice(), key) {
                        Ordering::Equal => return Some(&(*next)),
                        Ordering::Less => {
                            cur = next;
                            continue;
                        }
                        Ordering::Greater => {
                            if level == 0 {
                                return Some(&(*next));
                            }
                        }
                    }
                }
            }
            if level == 0 { 
                break;
            }
            level -= 1;
        }
        unsafe {
            if cur.is_null() || cur == self.head.as_ref() {
                None
            } else if self.cmp.cmp((*cur).key.as_slice(), key) == Ordering::Less {
                None
            } else {
                Some(&(*cur))
            }
        }
    }

    /// return if the skiplist contains the key
    fn contains(&self, key: &[u8]) -> bool {
        if let Some(n) = self.get_greater_or_equal(key) {
            self.cmp.cmp(n.key.as_slice(), key) == Ordering::Equal
        } else {
            false
        }
    }

    /// get the last smaller node
    fn get_last_smaller<'a>(&'a self, key: &[u8]) -> Option<&'a Node> {
        let mut cur = self.head.as_ref() as *const Node;
        let mut level = self.head.skips.len() - 1;

        loop {
            unsafe {
                if let Some(next) = (*cur).skips[level] {
                    if self.cmp.cmp((*next).key.as_slice(), key) == Ordering::Less {
                        cur = next;
                        continue;
                    }
                }
                if level == 0 {
                    break;
                }
                level -= 1;
            }
        }

        unsafe {
            if cur.is_null() || cur == self.head.as_ref() {
                None
            } else if self.cmp.cmp((*cur).key.as_slice(), key) != Ordering::Less {
                None
            }
            else {
                Some(&(*cur))
            }
        }
    }

    fn insert(&mut self, key: &[u8], val: &[u8]) {
        let new_height = self.random_height();
        let mut level = MAX_HEIGHT - 1;
        let mut current = self.head.as_mut() as *mut Node;
        let mut prevs = vec![Some(current); new_height];

        loop {
            unsafe {
                if let Some(next) = (*current).skips[level] {
                    let ord =  self.cmp.cmp((*next).key.as_slice(), key);
                    assert!(ord != Ordering::Equal);
                    if ord == Ordering::Less {
                        current = next;
                        continue;
                    } 
                }
            }
            if level < new_height {
                prevs[level] = Some(current);
            }
            if level == 0 {
                break;
            }
            level -= 1;
        }

        let mut new_node = Box::new(Node {
            key: key.to_vec(),
            value: val.to_vec(),
            next: None,
            skips: vec![None; new_height],
        });
        for (i, prev) in prevs.iter().enumerate() {
            unsafe {
                if let Some(prev) = prev {
                    new_node.skips[i] = (*(*prev)).skips[i];
                    (*(*prev)).skips[i] = Some(new_node.as_mut() as *mut Node);
                }
            }
        }

        let added_mem = size_of::<Node>() + size_of::<Option<*mut Node>>() * new_node.skips.len() + new_node.key.len() + new_node.value.len();
        self.approx_mem += added_mem;
        self.len += 1;

        unsafe {
            new_node.next = (*current).next.take();
            let _ = mem::replace(&mut(*current).next, Some(new_node));
        }
    }
}

pub struct SkipList {
    skip_list: Rc<RefCell<InnerSkipList>>,
}

impl SkipList {
    fn new(cmp: Rc<Box<dyn Cmp>>) -> SkipList {
        SkipList {
            skip_list: Rc::new(RefCell::new(InnerSkipList {
                head: Box::new(Node {
                    key: Vec::new(),
                    value: Vec::new(),
                    next: None,
                    skips: vec![None; MAX_HEIGHT],
                }),
                len: 0,
                approx_mem: size_of::<Self>() + MAX_HEIGHT * size_of::<Option<*mut Node>>(),
                rand: StdRng::seed_from_u64(0xdeadbeaf),
                cmp,
            })),
        }
    }

    pub fn len(&self) -> usize {
        self.skip_list.borrow().len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn approx_memory(&self) -> usize {
        self.skip_list.borrow().approx_mem
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        self.skip_list.borrow().contains(key)
    }

    pub fn insert(&self, key: &[u8], val: &[u8]) {
        assert!(!key.is_empty());
        self.skip_list.borrow_mut().insert(key, val);
    }

    pub fn iter(&self) -> SkipListIter {
        SkipListIter {
            skip_list: self.skip_list.clone(),
            cur: self.skip_list.borrow().head.as_ref() as *const Node,
        }
    }
}

pub struct SkipListIter {
    skip_list: Rc<RefCell<InnerSkipList>>,
    cur: *const Node,
}

impl DBIter for SkipListIter {
    fn next(&mut self) -> Option<(&[u8], &[u8])> {
        unsafe {
            if self.cur.is_null() {
                return None
            }
            (*self.cur).next.as_ref().map(|n| {
                self.cur = n.as_ref() as *const Node;
                (n.key.as_slice(), n.value.as_slice())
            })
        }
    }

    fn peek(&self) -> Option<(&[u8], &[u8])> {
        unsafe {
            if self.cur.is_null() {
                return None
            }
            (*self.cur).next.as_ref().map(|n| {
                (n.key.as_slice(), n.value.as_slice())
            })
        }
    }

    fn seek(&mut self, key: &[u8]) {
        if let Some(c) = self.skip_list.borrow().get_greater_or_equal(key).map(|n| n as *const Node) {
            self.cur = c;
        } else {
            self.cur = ptr::null()
        }
    }

    fn reset(&mut self) {
        self.cur = self.skip_list.borrow().head.as_ref() as *const Node;
    }

    fn prev(&mut self) -> Option<(&[u8], &[u8])> {
        if self.cur.is_null() {
            return None
        } else {
            unsafe {
                let key = (*self.cur).key.as_slice();
                if let Some(node) = self.skip_list.borrow().get_last_smaller(key) {
                    self.cur = node as *const Node;
                    return self.peek();
                } else {
                    return None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cmp;

    use super::*;

    fn make_skiplist() -> SkipList {
        let mut skip_list = SkipList::new(Rc::new(Box::new(cmp::DefaultCmp)));
        let keys = vec![
            "aba", "abb", "abc", "abd", "abe", "abf", "abg", "abh", "abi", "abj", "abk", "abl",
            "abm", "abn", "abo", "abp", "abq", "abr", "abs", "abt", "abu", "abv", "abw", "abx",
            "aby", "abz",
        ];
        for k in keys {
            skip_list.insert(k.as_bytes(), "def".as_bytes())
        }
        skip_list
    }

    #[test]
    #[should_panic]
    fn no_duplicate() {
        let sl = make_skiplist();
        sl.insert("aba".as_bytes(), "def".as_bytes());
        sl.insert("abd".as_bytes(), "def".as_bytes());
    }

    #[test]
    fn test_contains() {
        let sl = make_skiplist();
        assert!(sl.contains("aba".as_bytes()));
        assert!(sl.contains("abb".as_bytes()));
        assert!(sl.contains("abc".as_bytes()));
        assert!(sl.contains("abd".as_bytes()));
        assert!(sl.contains("abe".as_bytes()));
        assert!(!sl.contains("def".as_bytes()));
    }
}