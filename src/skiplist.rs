use std::{cmp::Ordering, rc::Rc};
use rand::{rngs::StdRng, RngCore};

use crate::cmp::Cmp;

const MAX_HEIGHT: usize = 12;
const BRANCHING_FACTOR: u32 = 4;

/// the internal node for inter skip list
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
}