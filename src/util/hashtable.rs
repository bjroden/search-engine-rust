use std::{num::ParseIntError, ops::AddAssign};

use sha2::{Sha256, Digest};
use hex;

pub fn hash_function(key: &str, size: &usize) -> Result<usize, ParseIntError> {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let hash_bytes = hasher.finalize();
    let hash_str = hex::encode(hash_bytes);
    let hash_slice = &hash_str[hash_str.len() - 7..];
    let result = usize::from_str_radix(hash_slice, 16)?;
    Ok(result % size)
}

pub fn rehash(hash: &usize, size: &usize) -> usize {
    (hash + 3) % size
}

#[derive(Clone)]
pub struct TableEntry<T> {
    pub key: String,
    pub value: T
}

pub struct HashTable<T> {
    size: usize,
    unique_tokens: usize,
    total_tokens: usize,
    buckets: Vec<Option<TableEntry<T>>>
}

impl<T> HashTable<T>
where T: Clone + AddAssign
{
    pub fn new(size: usize) -> Self {
        Self { 
            size: size, 
            unique_tokens: 0, 
            total_tokens: 0, 
            buckets: vec![None; size]
        }
    }

    pub fn reset(&mut self) {
        self.unique_tokens = 0;
        self.total_tokens = 0;
        for bucket in &mut self.buckets {
            *bucket = None;
        }
    }

    pub fn insert_combine(&mut self, key: &str, value: T) {
        if let Ok(mut hash) = hash_function(key, &self.size) {
            while let Some(Some(bucket)) = self.buckets.get(hash) {
                if bucket.key != key { hash = rehash(&hash, &self.size)}
                else { break }
            }
            match self.buckets.get_mut(hash) {
                Some(Some(bucket)) =>  { 
                    bucket.value += value;
                    self.total_tokens += 1;
                    self.unique_tokens += 1;
                }
                Some(None) =>  { 
                    self.buckets[hash] = Some(TableEntry { key: key.to_string(), value: value });
                    self.total_tokens += 1;
                }
                _ => ()
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        if let Ok(mut hash) = hash_function(key, &self.size) {
            while let Some(Some(bucket)) = self.buckets.get(hash) {
                if bucket.key == key { return Some(&bucket.value) }
                else { hash = rehash(&hash, &self.size) }
            }
        }
        None
    }

    pub fn intable(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn get_buckets(&self) -> &Vec<Option<TableEntry<T>>> {
        return &self.buckets
    }

}