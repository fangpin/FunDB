use std::cmp::Ordering;

use crate::cmp::Cmp;

#[derive(Clone)]
pub enum ValueType {
    TypeDeletion = 0,
    TypeValue = 1,
}

pub type SeqNum = u64;

/// the actual key passed from the client
pub type UserKey<'a> = &'a [u8];

/// [UserKey + SeqNum(7 bytes) + ValueType(1 byte)]
pub type InternalKey<'a> = &'a [u8];

/// [key_len + InternalKey + value_len + value]
pub type MemKey<'a> = &'a [u8];

/// [key_len + InteralKey] (first part of mem key)
pub struct LookupKey {
    key: Vec<u8>,
}

const U64_SIZE: usize = 8;
const U32_SIZE: usize = 4;

#[inline]
fn u32_to_bytes(num: u32) -> Vec<u8> {
    let mut ret = Vec::with_capacity(4);
    for i in 0..4 { 
        ret.push((num >> (i * 8)) as u8)
    }
    ret
}

#[inline]
fn u64_to_bytes(num: u64) -> Vec<u8> {
    let mut ret = Vec::with_capacity(8);
    for i in 0..8 { 
        ret.push((num >> (i * 8)) as u8)
    }
    ret
}

#[inline]
fn u32_from_bytes(bytes: &[u8]) -> u32 {
    let mut ret = 0;
    for i in 0..4 {
        ret |= (bytes[i] as u32) << (i * 8);
    }
    ret
}

#[inline]
fn u64_from_bytes(bytes: &[u8]) -> u64 {
    let mut ret = 0;
    for i in 0..8 {
        ret |= (bytes[i] as u64) << (i * 8);
    }
    ret
}

impl LookupKey {
    pub fn new(key: &[u8], seq: u64, v_type: ValueType) -> Self {
        let mut tag: u64 = (seq as u64) << 1 | (v_type as u64);

        let mut vec = Vec::with_capacity(U32_SIZE + key.len() + U64_SIZE);
        vec.extend_from_slice(u32_to_bytes(key.len() as u32).as_slice());
        vec.extend_from_slice(key);
        vec.extend_from_slice(u64_to_bytes(tag).as_slice());

        LookupKey { key: vec }
    }

    pub fn mem_key(&self) -> MemKey {
        &self.key
    }

    pub fn user_key(&self) -> UserKey {
        &self.key[U32_SIZE..self.key.len() - U64_SIZE]
    }
    
    pub fn internal_key(&self) -> InternalKey {
        &self.key[U32_SIZE..self.key.len()] 
    }
}

/// parse the tag, and return the seq num and value type
pub fn parse_tag(tag: u64) -> (u64, ValueType) {
    let seq = tag >> 8;
    let typ = tag & 0xff;
    match typ {
        0 => (seq, ValueType::TypeDeletion),
        1 => (seq, ValueType::TypeValue),
        _ => panic!("invalid tag: {}", tag),
    }
}

#[inline]
pub fn build_tag(seq: &SeqNum, typ: &ValueType) -> u64 {
    (seq << 8) | typ.clone() as u64
}

pub fn build_mem_key(key: &[u8], value: &[u8], seq: &SeqNum, typ: &ValueType) -> Vec<u8> {
    let key_size = key.len();
    let value_size = value.len();
    let mut vec = Vec::with_capacity(U32_SIZE + key_size + U64_SIZE + U32_SIZE + value_size);

    vec.extend_from_slice(u32_to_bytes(key_size as u32).as_slice());
    vec.extend_from_slice(key);
    vec.extend_from_slice(u64_to_bytes(build_tag(seq, typ)).as_slice());
    vec.extend_from_slice(u32_to_bytes(value_size as u32).as_slice());
    vec.extend_from_slice(value);

    vec
}

pub fn parse_mem_key(key: MemKey) -> (&[u8], SeqNum, ValueType, &[u8]) {
    let key_size = u32_from_bytes(&key[0..U32_SIZE]);
    let k = &key[U32_SIZE..key_size as usize + U32_SIZE];
    let key_end = key_size as usize + U32_SIZE;
    let value_start = key_end + U64_SIZE + U32_SIZE;
    let tag = u64_from_bytes(&key[key_end..key_end + U64_SIZE]);
    let value_len = u32_from_bytes(&key[key_end + U64_SIZE..key_end + U64_SIZE + U32_SIZE]);
    let (seq, typ) = parse_tag(tag);    
    let v = &key[value_start..value_start + value_len as usize];
    (k, seq, typ, v)
}

/// compare the mem key by parsing and comparing the user key. If user key is equal, compare the seq num.
pub fn cmp_mem_key(ucmp: &dyn Cmp, a: MemKey, b: MemKey) -> Ordering {
    let (a_user_key, a_seq,_, _) = parse_mem_key(a);
    let (b_user_key, b_seq, _, _)= parse_mem_key(b);
    match ucmp.cmp(a_user_key, b_user_key) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => b_seq.cmp(&a_seq), // to make sure the key with higher seq number can be found firstly when in lower bound search
    }
}

/// parse the internal key
pub fn parse_internal_key(ikey: InternalKey) -> (&[u8], SeqNum, ValueType) {
    if ikey.len() == 0 {
        return (&ikey[0..], 0, ValueType::TypeDeletion);
    }
    debug_assert!(ikey.len() >= U64_SIZE);
    let key_end = ikey.len() - U64_SIZE;
    let key = &ikey[0..key_end];
    let (seq, typ) = parse_tag(u64_from_bytes(&ikey[key_end..]));
    (key, seq, typ)
}

/// compare internal key
pub fn cmp_internal_key(ucmp: &dyn Cmp, a: InternalKey, b: InternalKey) -> Ordering {
    let (a_internal_key, a_seq, _) = parse_internal_key(a);
    let (b_internal_key, b_seq, _) = parse_internal_key(b);

    match ucmp.cmp(a_internal_key, b_internal_key) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => b_seq.cmp(&a_seq),
    }
}

/// truncate the internal key to user key
pub fn truncate_internal_to_user_key(ikey: InternalKey) -> UserKey {
    let len = ikey.len();
    debug_assert!(len >= U64_SIZE);
    &ikey[..len - U64_SIZE]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memtable_lookupkey() {
        let lk1 = LookupKey::new("abcde".as_bytes(), 123, ValueType::TypeValue);
        let lk2 = LookupKey::new("xyabxy".as_bytes(), 97, ValueType::TypeValue);

        // Assert correct allocation strategy
        assert_eq!(lk1.key.len(), 17);
        assert_eq!(lk2.key.len(), 18);

        assert_eq!(lk1.user_key(), "abcde".as_bytes());
    }

    #[test]
    fn test_build_memtable_key() {
        let mem_key = build_mem_key(
            "abc".as_bytes(),
            "123".as_bytes(),
            &231,
            &ValueType::TypeValue,
        );

        let (user_key, seq, typ, value) = parse_mem_key(&mem_key);
        assert_eq!(user_key, "abc".as_bytes());
        assert_eq!(value, "123".as_bytes());
        assert_eq!(seq, 231);
        assert_eq!(typ.clone() as u32, ValueType::TypeValue.clone() as u32);

        let mem_key2 = build_mem_key(user_key, value, &seq, &typ);
        assert_eq!(mem_key, mem_key2);
    }
}