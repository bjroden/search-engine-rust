use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::{File, self};
use std::io::{Error, BufReader, Seek, SeekFrom, BufRead};

use std::vec;

use crate::parser::parse;
use crate::data_models::{DictRecord, PostRecord, NamedResult, FileSizes};
use crate::hashtable::{hash_function, rehash, HashTable};

fn get_query_tokens(query: &str) -> Vec<String> {
    return parse(query);
}

fn get_sizes(filedir: &str) -> Result<FileSizes, Error> {
    let file_contents = fs::read_to_string(format!("{filedir}/sizes"))?;
    let sizes: FileSizes = serde_json::from_str(&file_contents)?;
    Ok(sizes)
}

fn get_dict_records(filedir: &str, tokens: &Vec<String>, sizes: &FileSizes) -> Result<Vec<DictRecord>, Error> {
    let mut records = vec![];
    let file = File::open(format!("{filedir}/dict"))?;
    let mut reader = BufReader::new(file);
    for token in tokens {
        if let Some(record) = get_one_dict_record(&mut reader, token, &sizes)? {
            records.push(record);
        }
    }
    Ok(records)
}

fn get_one_dict_record(reader: &mut BufReader<File>, token: &str, sizes: &FileSizes) -> Result<Option<DictRecord>, Error> {
    let mut hash = hash_function(token, &sizes.num_dict_lines).unwrap();
    let mut record = read_one_dict_line_from_hash(reader, &sizes, hash)?;
    while record.term != "!NULL" && record.term != token { 
        hash = rehash(&hash, &sizes.num_dict_lines);
        record = read_one_dict_line_from_hash(reader, &sizes, hash)?;
    }
    if record.term.starts_with("!") { return Ok(None); }
    Ok(Some(record))
}

fn read_one_dict_line_from_hash(reader: &mut BufReader<File>, sizes: &FileSizes, hash: usize) -> Result<DictRecord, Error> {
    reader.seek(SeekFrom::Start((hash * sizes.get_dict_record_size()).try_into().unwrap()))?;
    let mut record_str = String::new();
    reader.read_line(&mut record_str)?;
    let split_record: Vec<&str> = record_str.split_whitespace().collect();
    let term = split_record[0];
    let num_docs = split_record[1].parse().unwrap();
    let start = split_record[2].parse().unwrap();
    Ok(DictRecord { term: term.to_string(), num_docs: num_docs, post_line_start: start })
}

fn make_query_ht(post_records: &Vec<PostRecord>, expected_docs: usize) -> HashTable<usize> {
    let mut query_ht = HashTable::new(expected_docs * 3);
    for record in post_records {
        query_ht.insert_combine(&record.doc_id.to_string(), record.weight);
    }
    query_ht
}

fn get_all_post_records(filedir: &str, dict_records: &Vec<DictRecord>, sizes: &FileSizes) -> Result<Vec<PostRecord>, Error> {
    let file = File::open(format!("{filedir}/post"))?;
    let mut reader = BufReader::new(file);
    let mut post_records = vec![];
    for dict_record in dict_records {
        post_records.append(&mut get_term_post_records(&mut reader, &dict_record, sizes)?);
    }
    Ok(post_records)
}

fn get_term_post_records(reader: &mut BufReader<File>, dict_record: &DictRecord, sizes: &FileSizes) -> Result<Vec<PostRecord>, Error> {
    let mut post_records = vec![];
    reader.seek(SeekFrom::Start((dict_record.post_line_start * sizes.get_post_record_size()).try_into().unwrap()))?;
    for _ in 0..dict_record.num_docs {
        let mut record_str = String::new();
        reader.read_line(&mut record_str)?;
        let split_record: Vec<&str> = record_str.split_whitespace().collect();
        let doc_id = split_record[0].parse().unwrap();
        let weight: usize = split_record[1].parse().unwrap();
        post_records.push(PostRecord { doc_id: doc_id, weight: weight })
    }
    Ok(post_records)
}

fn get_sorted_results(query_ht: HashTable<usize>, num_results: usize) -> Vec<PostRecord> {
    let mut heap: BinaryHeap<Reverse<PostRecord>> = BinaryHeap::new();
    for bucket in query_ht.get_buckets() {
        if let Some(entry) = bucket {
            match heap.peek() {
                Some(Reverse(heap_head)) => {
                    if heap.len() < num_results {
                        heap.push(Reverse(PostRecord { doc_id: entry.key.parse().unwrap(), weight: entry.value }));
                    }
                    else if heap_head.weight < entry.value {
                        heap.pop();
                        heap.push(Reverse(PostRecord { doc_id: entry.key.parse().unwrap(), weight: entry.value }));
                    }
                }
                None => heap.push(Reverse(PostRecord { doc_id: entry.key.parse().unwrap(), weight: entry.value }))
            }
        }
    }
    let rev_sorted = heap.into_sorted_vec();
    let mut sorted = vec![];
    for Reverse(elem) in rev_sorted { sorted.push(elem); }
    sorted
}

fn get_named_results(filedir: &str, results: Vec<PostRecord>, sizes: &FileSizes) -> Result<Vec<NamedResult>, Error> {
    let mut named_results = vec![];
    let file = File::open(format!("{filedir}/map"))?;
    let mut reader = BufReader::new(file);
    for result in results {
        named_results.push(NamedResult { name: get_doc_name(&mut reader, result.doc_id, sizes)?, weight: result.weight })
    }
    Ok(named_results)
}

fn get_doc_name(reader: &mut BufReader<File>, doc_id: usize, sizes: &FileSizes) -> Result<String, Error> {
    let mut name = String::new();
    reader.seek(SeekFrom::Start((doc_id * sizes.get_map_record_size()).try_into().unwrap()))?;
    reader.read_line(&mut name)?;
    Ok(name.trim().to_string())
}

pub fn make_query(query: &str, filedir: &str, num_results: usize) -> Result<Vec<NamedResult>, Error> {
    let sizes = get_sizes(filedir)?;
    let tokens = get_query_tokens(query);
    let dict_records = get_dict_records(filedir, &tokens, &sizes)?;
    let expected_docs = dict_records.iter().fold(0, |sum, record| sum + record.num_docs);
    let post_records = get_all_post_records(filedir, &dict_records, &sizes)?;
    let query_ht = make_query_ht(&post_records, expected_docs);
    let sorted_results = get_sorted_results(query_ht, num_results);
    get_named_results(filedir, sorted_results, &sizes)
}