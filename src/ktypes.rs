use std::io::Write;

// Order matters. Make sure deleted value can be ordered before any other value.
pub enum ValueType {
    TypeDeletion = 0,
    TypeValue = 1,
}

/// the actual key passed from the client
pub type UserKey<'a> = &'a [u8];

/// [UserKey + SeqNum(7 bit) + ValueType(1 bit)]
pub type InternalKey<'a> = &'a [u8];

/// [key_len + InternalKey + value_len + value]
pub type MemKey<'a> = &'a [u8];

/// [key_len + InteralKey]
pub struct LookupKey {
    key: Vec<u8>,
}

const U64_SIZE: usize = 8;
const SIZE_SIZE: usize = 4;

#[inline]
pub fn u32_to_bytes(num: u32) -> Vec<u8> {
    let mut ret = Vec::with_capacity(4);
    for i in 0..4 { 
        ret.push((num >> (i * 8)) as u8)
    }
    ret
}

#[inline]
pub fn u64_to_bytes(num: u64) -> Vec<u8> {
    let mut ret = Vec::with_capacity(8);
    for i in 0..8 { 
        ret.push((num >> (i * 8)) as u8)
    }
    ret
}

#[inline]
pub fn u32_from_bytes(bytes: &[u8]) -> u32 {
    let mut ret = 0;
    for i in 0..4 {
        ret |= (bytes[i] as u32) << (i * 8);
    }
    ret
}

#[inline]
pub fn u64_from_bytes(bytes: &[u8]) -> u64 {
    let mut ret = 0;
    for i in 0..8 {
        ret |= (bytes[i] as u64) << (i * 8);
    }
    ret
}

impl LookupKey {
    pub fn new(key: &Vec<u8>, seq: u64, v_type: ValueType) -> Self {
        let mut tag: u64 = (seq as u64) << 1 | (v_type as u64);

        let mut vec = Vec::with_capacity(SIZE_SIZE + key.len() + U64_SIZE);
        vec.extend_from_slice(u32_to_bytes(key.len() as u32).as_slice());
        vec.extend_from_slice(key);
        vec.extend_from_slice(u64_to_bytes(tag).as_slice());

        LookupKey { key: vec }
    }

    pub fn mem_key(&self) -> MemKey {
        &self.key
    }

    pub fn user_key(&self) -> UserKey {
        &self.key[SIZE_SIZE..self.key.len() - U64_SIZE]
    }
    
    pub fn internal_key(&self) -> InternalKey {
        &self.key[SIZE_SIZE..self.key.len()] 
    }
}