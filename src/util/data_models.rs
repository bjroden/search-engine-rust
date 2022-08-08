use std::ops::AddAssign;

#[derive(Clone)]
pub struct PostRecord {
    doc_id: usize,
    rtf: usize
}

pub struct DictRecord {
    term: String,
    num_docs: usize,
    post_line_start: usize
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
    pub fn new(doc_id: usize, rtf: usize) -> Self {
        Self { files: vec![PostRecord { doc_id, rtf }] }
    }

    pub fn get_num_docs(&self) -> usize {
        self.files.len()
    }

    pub fn get_total_frequency(&self) -> usize {
        self.files.iter().fold(0, |sum, file| sum + file.rtf)
    }
}

impl AddAssign for GlobHTBucket {
    fn add_assign(&mut self, rhs: Self) {
        self.files.append(&mut rhs.files.clone())
    }
}