use std::{ops::AddAssign, cmp};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use crate::{constants::*, hashtable::HashTable};

pub struct NamedResult {
    pub name: String,
    pub weight: usize
}

#[derive(Clone)]
pub struct DocFrequency {
    pub doc_id: usize,
    pub raw_term_frequency: usize,
    pub relative_term_frequency: f64
}

#[derive(Eq, Clone)]
pub struct PostRecord {
    pub doc_id: usize,
    pub weight: usize
}

impl Ord for PostRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl PartialOrd for PostRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.weight.cmp(&other.weight))
    }
}

impl PartialEq for PostRecord {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

pub struct DictRecord {
    pub term: String,
    pub num_docs: usize,
    pub post_line_start: usize
}

pub struct MapRecord {
    pub doc_id: usize,
    pub file_name: String
}

#[derive(Serialize, Deserialize)]
pub struct FileSizes {
    pub num_dict_lines: usize,
    pub post_line_start_length: usize,
    pub num_docs_length: usize,
    pub doc_id_length: usize
}

impl FileSizes {
    pub fn new(glob_ht: &HashTable<GlobHTBucket>, map_files: &Vec<MapRecord>) -> Self {
        Self {
            num_dict_lines: glob_ht.get_size(),
            post_line_start_length: Self::calculate_post_line_start_length(&glob_ht),
            num_docs_length: Self::calculate_num_docs_length(&glob_ht),
            doc_id_length: map_files.len().to_string().len()
        }
    }

    pub fn get_dict_record_size(&self) -> usize {
        TERM_LENGTH + self.num_docs_length + self.post_line_start_length + 3
    }

    pub fn get_post_record_size(&self) -> usize {
        self.doc_id_length + WEIGHT_LENGTH + 2
    }

    fn calculate_num_docs_length(glob_ht: &HashTable<GlobHTBucket>) -> usize {
        let num_docs_max = glob_ht.get_buckets().iter().fold(0, |max, bucket| cmp::max(max, match bucket {
            Some(entry) => entry.value.get_num_docs(),
            None => 0 
        }));
        num_docs_max.to_string().len()
    }

    fn calculate_post_line_start_length(glob_ht: &HashTable<GlobHTBucket>) -> usize {
        let num_post_records = glob_ht.get_buckets().iter().fold(0, |sum, bucket| sum + match bucket {
            Some(entry) => entry.value.get_files().len(),
            None => 0
        });
        num_post_records.to_string().len()
    }
}

#[derive(Clone)]
pub struct GlobHTBucket {
    files: Vec<DocFrequency>
}

impl GlobHTBucket {
    pub fn new(doc_id: usize, raw_term_frequency: usize, relative_term_frequency: f64) -> Self {
        Self { files: vec![DocFrequency { doc_id, raw_term_frequency, relative_term_frequency }] }
    }

    pub fn get_num_docs(&self) -> usize {
        self.files.len()
    }

    pub fn get_total_frequency(&self) -> usize {
        self.files.iter().fold(0, |sum, file| sum + file.raw_term_frequency)
    }

    pub fn get_files(&self) -> &Vec<DocFrequency> {
        &self.files
    }

    pub fn is_rare(&self) -> bool {
        self.get_num_docs() <= 1 && self.get_total_frequency() <= 1
    }
}

impl AddAssign for GlobHTBucket {
    fn add_assign(&mut self, rhs: Self) {
        self.files.append(&mut rhs.files.clone());
    }
}