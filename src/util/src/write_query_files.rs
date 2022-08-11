use std::{fs::{File, OpenOptions}, io::{Error, Write, BufWriter}};

use crate::{data_models::{GlobHTBucket, DocFrequency, MapRecord, FileSizes}, hashtable::{TableEntry, HashTable}};
use crate::constants::*;

fn write_dict(outdir: &str, glob_ht: &HashTable<GlobHTBucket>) -> Result<(), Error> {
    let dict_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/dict"))?;
    let mut writer = BufWriter::new(dict_file);
    let mut count: usize = 0;
    for bucket in glob_ht.get_buckets() {
        count = write_dict_line(&mut writer, bucket, count)?;
    }
    Ok(())
}

fn write_dict_line(writer: &mut BufWriter<File>, bucket: &Option<TableEntry<GlobHTBucket>>, count: usize) -> Result<usize, Error> {
    let mut new_count = count;
    let (term, num_docs, post_line_start) = match bucket {
        Some(entry) => {
            if entry.value.is_rare() {
                ("!DELETED", 0, 0)
            } 
            else {
                new_count += entry.value.get_num_docs();
                (entry.key.as_str(), entry.value.get_num_docs(), count)
            }
        }
        None => ("!NULL", 0, 0)
    };
    writeln!(writer, 
            "{:<term_length$.term_length$} {:<num_docs_length$.num_docs_length$} {:<start_length$.start_length$}",
            term, num_docs.to_string(), post_line_start.to_string(),
            term_length = TERM_LENGTH,
            num_docs_length = NUMDOCS_LENGTH,
            start_length = START_LENGTH
    )?;
    Ok(new_count)
}

fn write_post(outdir: &str, glob_ht: &HashTable<GlobHTBucket>, total_docs: usize) -> Result<(), Error> {
    let post_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/post"))?;
    let mut writer = BufWriter::new(post_file);
    for bucket in glob_ht.get_buckets() {
        if let Some(entry) = bucket {
            if !entry.value.is_rare() {
                let idf = 1.0 + (total_docs as f64 / entry.value.get_num_docs() as f64).log10();
                for file in entry.value.get_files() {
                    write_post_line(&mut writer, file, idf)?;
                }
            }
        }
    }
    Ok(())
}

fn write_post_line(writer: &mut BufWriter<File>, file: &DocFrequency, idf: f64) -> Result<(), Error> {
    let doc_id = file.doc_id;
    let weight = (file.relative_term_frequency * idf * WEIGHT_MULTIPLIER) as usize; 
    writeln!(writer,
        "{:<doc_id_length$.doc_id_length$} {:<weight_length$.weight_length$}",
        doc_id.to_string(), weight.to_string(),
        doc_id_length = DOC_ID_LENGTH,
        weight_length = WEIGHT_LENGTH
    )?;
    Ok(())
}


fn write_map(outdir: &str, docs: &Vec<MapRecord>) -> Result<(), Error> {
    let map_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/map"))?;
    let mut writer = BufWriter::new(map_file);
    for doc in docs {
        write_map_line(&mut writer, &doc.file_name)?;
    }
    Ok(())
}

fn write_map_line(writer: &mut BufWriter<File>, name: &str) -> Result<(), Error> {
    writeln!(writer, "{:<length$.length$}", name, length=MAP_NAME_LENGTH)?;
    Ok(())
}

fn write_sizes(outdir: &str, glob_ht: &HashTable<GlobHTBucket>) -> Result<(), Error> {
    let sizes_file = OpenOptions::new().write(true).create(true).truncate(true).open(format!("{outdir}/sizes"))?;
    let mut writer = BufWriter::new(sizes_file);
    let sizes = serde_json::to_string(&FileSizes { num_dict_lines: glob_ht.get_size() })?;
    writeln!(&mut writer, "{}", sizes)?;
    Ok(())
}

pub fn write_output_files(outdir: &str, glob_ht: &HashTable<GlobHTBucket>, map_files: &Vec<MapRecord>) -> Result<(), Error> {
    write_dict(&outdir, &glob_ht)?;
    write_post(&outdir, &glob_ht, map_files.len())?;
    write_map(&outdir, &map_files)?;
    write_sizes(&outdir, &glob_ht)?;
    Ok(())
}
