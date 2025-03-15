use actix_web::web;
use handlebars::Handlebars;
use std::{env, fs};
use std::io;
use std::path::{Path, PathBuf};

pub fn register(cfg: &mut web::ServiceConfig) {
    let tmpl: Handlebars = make();

    cfg.app_data(web::Data::new(tmpl));
}

pub fn make() -> Handlebars<'static> {
    let mut tmpl: Handlebars = Handlebars::new();

    let mut handlebars_dir = env::current_dir().unwrap();
    handlebars_dir.push(Path::new("resources/handlebars"));
    let str_handlebars_dir = handlebars_dir.to_owned();
    let str_handlebars_dir = str_handlebars_dir.to_str().unwrap();

    let collect_paths: Vec<PathBuf> =
        collect_files_from_dir(handlebars_dir.as_path()).unwrap();
    let paths: Vec<&PathBuf> = collect_paths
        .iter()
        .filter(|&p| {
            p.extension().unwrap() == "hbs"
                || p.extension().unwrap() == "handlebars"
                || p.extension().unwrap() == "html"
        })
        .collect::<Vec<&PathBuf>>();

    for path in paths {
        let str_path = path.to_str().unwrap();
        let replace_str = format!("{}/", str_handlebars_dir);
        let replace_str = replace_str.as_str();
        let name = str_path.replace(replace_str, "");

        // Register_template_string
        // let content = fs::read_to_string(str_path).unwrap();
        // tmpl.register_template_string(name.as_str(), content)
        //     .unwrap();

        // Register_template_file
        tmpl.register_template_file(name.as_str(), path).unwrap();
    }

    tmpl
}

fn collect_files_from_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut result: Vec<PathBuf> = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                result.extend(collect_files_from_dir(&path)?);
            } else {
                result.push(path);
            }
        }
    }
    Ok(result)
}
