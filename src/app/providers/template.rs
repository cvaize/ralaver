use actix_web::web;
use handlebars::Handlebars;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn register(cfg: &mut web::ServiceConfig) {
    let mut tmpl: Handlebars = Handlebars::new();

    let collect_paths: Vec<PathBuf> =
        collect_files_from_dir(Path::new("./resources/handlebars")).unwrap();
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
        let name = str_path.replace("./resources/handlebars/", "");
        let content = fs::read_to_string(str_path).unwrap();

        tmpl.register_template_string(name.as_str(), content)
            .unwrap();
    }

    cfg.app_data(web::Data::new(tmpl));
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
