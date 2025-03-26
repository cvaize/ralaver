use crate::helpers::collect_files_from_dir;
use actix_web::web;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::{env, fs};
use serde_json::Value;
use crate::app::services::translates::Translates;

pub fn register(cfg: &mut web::ServiceConfig) {
    let tmpl: Translates = make().unwrap();

    cfg.app_data(web::Data::new(tmpl));
}

fn make() -> Result<Translates, io::Error> {
    let mut map: HashMap<String, String> = HashMap::from([]);

    let mut handlebars_dir = env::current_dir().unwrap();
    handlebars_dir.push(Path::new("resources/lang"));
    let str_handlebars_dir = handlebars_dir.to_owned();
    let str_handlebars_dir = str_handlebars_dir.to_str().unwrap();

    let collect_paths: Vec<PathBuf> = collect_files_from_dir(handlebars_dir.as_path()).unwrap();
    let paths: Vec<&PathBuf> = collect_paths
        .iter()
        .filter(|&p| p.extension().unwrap() == "json")
        .collect::<Vec<&PathBuf>>();

    for path in paths {
        let str_path = path.to_str().unwrap();
        let replace_str = format!("{}/", str_handlebars_dir);
        let replace_str = replace_str.as_str();
        let name = str_path
            .replace(replace_str, "")
            .replace(".json", "")
            .replace("/", ".");

        let content = fs::read_to_string(str_path).unwrap();

        let flat_json: String = flatten_json::flatten_from_str(&content).unwrap();
        let flatten_keys: Value = serde_json::from_str(&flat_json).unwrap();

        for (key, value) in flatten_keys.as_object().unwrap().iter() {
            let mut full_key: String = name.clone();
            full_key.push_str(".");
            full_key.push_str(key);
            if let Some(value) = value.as_str() {
                map.insert(full_key.to_string(), value.to_string());
            }
        }
    }

    Ok(Translates::new(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make() {
        let tmpl: Translates = super::make().unwrap();
        let translates: &HashMap<String, String> = tmpl.map();
        assert_ne!(0, translates.iter().len());
    }
}
