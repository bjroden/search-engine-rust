use rocket::fs::NamedFile;
use util::read_query_files::make_query;
use web_result::get_web_results;
use std::{env, path::{PathBuf, Path}};

mod web_result;

#[macro_use] extern crate rocket;

#[get("/files/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    let static_file_dir = env::var("STATIC_FILES_DIR").unwrap_or("static".to_string());
    NamedFile::open(Path::new(&static_file_dir).join(file)).await.ok()
}

#[get("/?<query>&<num_results>")]
fn index(query: Option<String>, num_results: Option<usize>) -> String {
    let query = query.unwrap_or("".to_string());
    let num_results = num_results.unwrap_or(10);
    let query_file_dir = env::var("QUERY_FILES_DIR").unwrap_or("query_files".to_string());
    let results = make_query(&query, &query_file_dir, num_results).unwrap();
    serde_json::to_string(&get_web_results(&results)).unwrap()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, files])
}
