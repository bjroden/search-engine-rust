use serde::Serialize;
use util::data_models::NamedResult;

#[derive(Serialize)]
pub struct WebResult<'a> {
    pub ranking: usize,
    pub name: &'a str,
    pub weight: usize
}

pub fn get_web_results(sorted_results: &Vec<NamedResult>) -> Vec<WebResult> {
    let mut json_results = vec![];
    for (rank, result) in sorted_results.iter().enumerate() {
        json_results.push(WebResult { ranking: rank + 1, name: &result.name, weight: result.weight })
    }
    json_results
}
