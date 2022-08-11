use std::{num::ParseIntError, ops::AddAssign, mem::take};

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
    buckets: Vec<Option<TableEntry<T>>>,
    num_elements: usize
}

impl<T> HashTable<T>
where T: Clone + AddAssign
{
    pub fn new(size: usize) -> Self {
        Self { 
            buckets: vec![None; size],
            num_elements: 0
        }
    }

    fn reindex(&mut self) {
        if self.num_elements * 2 > self.buckets.len() {
            let old_buckets = take(&mut self.buckets);
            self.buckets = vec![None; old_buckets.len() * 2];
            for bucket in old_buckets {
                if let Some(entry) = bucket {
                    self.reinsert_helper(entry);
                }
            }
        }
    }

    fn reinsert_helper(&mut self, entry: TableEntry<T>) {
        if let Ok(mut hash) = hash_function(&entry.key, &self.get_size()) {
            while let Some(Some(bucket)) = self.buckets.get(hash) {
                if bucket.key != entry.key { hash = rehash(&hash, &self.get_size())}
                else { break }
            }
            if let Some(None) = self.buckets.get_mut(hash) {
                self.buckets[hash] = Some(entry);
            }
        }
    }

    pub fn reset(&mut self) {
        for bucket in &mut self.buckets {
            *bucket = None;
        }
    }

    pub fn insert_combine(&mut self, key: &str, value: T) {
        self.reindex();
        if let Ok(mut hash) = hash_function(key, &self.get_size()) {
            while let Some(Some(bucket)) = self.buckets.get(hash) {
                if bucket.key != key { hash = rehash(&hash, &self.get_size())}
                else { break }
            }
            match self.buckets.get_mut(hash) {
                Some(Some(bucket)) =>  { 
                    bucket.value += value;
                }
                Some(None) =>  { 
                    self.buckets[hash] = Some(TableEntry { key: key.to_string(), value: value });
                    self.num_elements += 1;
                }
                _ => ()
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        if let Ok(mut hash) = hash_function(key, &self.get_size()) {
            while let Some(Some(bucket)) = self.buckets.get(hash) {
                if bucket.key == key { return Some(&bucket.value) }
                else { hash = rehash(&hash, &self.get_size()) }
            }
        }
        None
    }

    pub fn intable(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn get_buckets(&self) -> &Vec<Option<TableEntry<T>>> {
        &self.buckets
    }

    pub fn get_size(&self) -> usize {
        self.buckets.len()
    }

    pub fn get_num_elements(&self) -> usize {
        self.num_elements
    }

}