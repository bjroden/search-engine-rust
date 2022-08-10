use util::read_query_files::make_query;
use web_result::get_web_results;
use std::env;

mod web_result;

#[macro_use] extern crate rocket;

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
    rocket::build().mount("/", routes![index])
}
