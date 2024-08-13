use std::rc::Rc;
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
}