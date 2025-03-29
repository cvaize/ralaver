use crate::helpers::collect_files_from_dir;
use crate::Config;
use actix_web::web::Data;
use actix_web::{error, Error};
use handlebars::{handlebars_helper, Handlebars};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::{env, io};

pub struct TemplateService {
    config: Data<Config>,
    handlebars: Handlebars<'static>,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderError;

impl TemplateService {
    pub fn render<T: Serialize>(&self, name: &str, data: &T) -> Result<String, RenderError>
    {
        match name.ends_with(".hbs") || name.ends_with(".handlebars") || name.ends_with(".html") {
            true => self.handlebars.render(name, data).map_err(|_| RenderError),
            _ => Err(RenderError),
        }
    }

    pub fn render_throw_http<T: Serialize>(&self, name: &str, data: &T) -> Result<String, Error>
    {
        self.render(name, data)
            .map_err(|_| error::ErrorInternalServerError("Template error"))
    }

    pub fn new_from_files(config: Data<Config>) -> Result<Self, io::Error> {
        let mut handlebars: Handlebars = Handlebars::new();

        let mut dir = env::current_dir()?;
        dir.push(Path::new(&config.get_ref().template.handlebars.folder));
        let str_dir = dir.to_owned();
        let str_dir = str_dir.to_str().unwrap();

        let collect_paths: Vec<PathBuf> = collect_files_from_dir(dir.as_path())?;
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
            let replace_str = format!("{}/", str_dir);
            let replace_str = replace_str.as_str();
            let name = str_path.replace(replace_str, "");

            // Register_template_string
            // let content = fs::read_to_string(str_path).unwrap();
            // tmpl.register_template_string(name.as_str(), content)
            //     .unwrap();

            // Register_template_file
            handlebars
                .register_template_file(name.as_str(), path)
                .unwrap();
        }

        handlebars.register_helper("eq", Box::new(eq));

        Ok(TemplateService { config, handlebars })
    }
}

handlebars_helper!(eq: |*args| args[0].eq(args[1]));
