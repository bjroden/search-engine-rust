use std::ops::AddAssign;

#[derive(Clone)]
pub struct PostRecord {
    pub doc_id: usize,
    pub raw_term_frequency: usize,
    pub relative_term_frequency: f64
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

#[derive(Clone)]
pub struct GlobHTBucket {
    files: Vec<PostRecord>
}

impl GlobHTBucket {
    pub fn new(doc_id: usize, raw_term_frequency: usize, relative_term_frequency: f64) -> Self {
        Self { files: vec![PostRecord { doc_id, raw_term_frequency, relative_term_frequency }] }
    }

    pub fn get_num_docs(&self) -> usize {
        self.files.len()
    }

    pub fn get_total_frequency(&self) -> usize {
        self.files.iter().fold(0, |sum, file| sum + file.raw_term_frequency)
    }

    pub fn get_files(&self) -> &Vec<PostRecord> {
        &self.files
    }
}

impl AddAssign for GlobHTBucket {
    fn add_assign(&mut self, rhs: Self) {
        self.files.append(&mut rhs.files.clone());
    }
}