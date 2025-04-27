use crate::helpers::collect_files_from_dir;
use crate::Config;
use actix_web::web::Data;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Debug, Clone)]
pub struct TranslatorService {
    config: Data<Config>,
    pub translates: HashMap<String, HashMap<String, String>>,
}

impl TranslatorService {
    pub fn new(config: Data<Config>, translates: HashMap<String, HashMap<String, String>>) -> Self {
        Self { config, translates }
    }

    pub fn new_from_files(config: Data<Config>) -> Result<Self, io::Error> {
        let mut translates: HashMap<String, String> = HashMap::from([]);

        let mut dir = env::current_dir().map_err(|e| {
            log::error!(
                "{}",
                format!("TranslatorService::new_from_files - {:}", &e).as_str()
            );
            e
        })?;
        dir.push(Path::new(&config.get_ref().translator.translates_folder));
        let str_dir = dir.to_owned();
        let str_dir = str_dir.to_str().unwrap();

        let collect_paths: Vec<PathBuf> = collect_files_from_dir(dir.as_path()).map_err(|e| {
            log::error!(
                "{}",
                format!("TranslatorService::new_from_files - {:}", &e).as_str()
            );
            e
        })?;
        let paths: Vec<&PathBuf> = collect_paths
            .iter()
            .filter(|&p| p.extension().unwrap() == "json")
            .collect::<Vec<&PathBuf>>();

        for path in paths {
            let str_path = path.to_str().unwrap();
            let replace_str = format!("{}/", str_dir);
            let replace_str = replace_str.as_str();
            let name = str_path
                .replace(replace_str, "")
                .replace(".json", "")
                .replace("/", ".");

            let content = fs::read_to_string(str_path).map_err(|e| {
                log::error!(
                    "{}",
                    format!("TranslatorService::new_from_files - {:}", &e).as_str()
                );
                e
            })?;

            let flat_json: String = flatten_json::flatten_from_str(&content).map_err(|e| {
                log::error!(
                    "{}",
                    format!("TranslatorService::new_from_files - {:}", &e).as_str()
                );
                e
            })?;
            let flatten_keys: Value = serde_json::from_str(&flat_json).map_err(|e| {
                log::error!(
                    "{}",
                    format!("TranslatorService::new_from_files - {:}", &e).as_str()
                );
                e
            })?;

            for (key, value) in flatten_keys.as_object().unwrap().iter() {
                let mut full_key: String = name.clone();
                full_key.push_str(".");
                full_key.push_str(key);
                if let Some(value) = value.as_str() {
                    translates.insert(full_key.to_string(), value.to_string());
                }
            }
        }

        let mut save_translates: HashMap<String, HashMap<String, String>> = HashMap::from([]);
        for (key, value) in translates.iter() {
            let mut split: Vec<&str> = key.split(".").collect();
            let lang = split.get(0).unwrap().to_string();
            split.remove(0);
            let key = split.join(".");

            if let Some(ts) = save_translates.get_mut(&lang) {
                ts.insert(key, value.to_owned());
            } else {
                save_translates.insert(lang, HashMap::from([(key, value.to_owned())]));
            }
        }

        Ok(Self::new(config, save_translates))
    }

    pub fn insert(&mut self, lang: &str, key: String, value: String) {
        if let Some(ts) = self.translates.get_mut(lang) {
            ts.insert(key, value);
        } else {
            let mut map: HashMap<String, String> = HashMap::new();
            map.insert(key, value);
            self.translates.insert(lang.to_string(), map);
        }
    }

    pub fn get(&self, lang: &str, key: &str) -> Option<&String> {
        if let Some(ts) = self.translates.get(lang) {
            return ts.get(key);
        }
        None
    }

    fn v_key(&self, key: &str) -> String {
        let mut v_key = ":".to_string();
        v_key.push_str(key);
        v_key
    }

    fn apply_variables(&self, string: String, variables: Vec<TranslatorVariable>) -> String {
        let mut string: String = string;
        for variable in &variables {
            string = match variable {
                TranslatorVariable::Usize(key, value) => {
                    string.replace(self.v_key(key).as_str(), value.to_string().as_str())
                }
                TranslatorVariable::I32(key, value) => {
                    string.replace(self.v_key(key).as_str(), value.to_string().as_str())
                }
                TranslatorVariable::U64(key, value) => {
                    string.replace(self.v_key(key).as_str(), value.to_string().as_str())
                }
                TranslatorVariable::String(key, value) => {
                    string.replace(self.v_key(key).as_str(), value)
                }
            }
        }
        string
    }

    // Функция возвращает перевод по переданному языку. Если перевод не найден по переданному языку,
    // то функция возвращает перевод по языку по умолчанию (app.locale). А если нет перевода по умолчанию, то берётся fallback язык (app.fallback_locale).
    // Если нет переводов с fallback языком, то возвращается переданный ключ.
    pub fn translate(&self, lang: &str, key: &str) -> String {
        if let Some(translate) = self.get(lang, key) {
            return translate.to_string();
        }

        let app = &self.config.get_ref().app;
        if lang != app.locale {
            if let Some(translate) = self.get(&app.locale, key) {
                return translate.to_string();
            }
        }

        if lang != app.fallback_locale && app.locale != app.fallback_locale {
            if let Some(translate) = self.get(&app.fallback_locale, key) {
                return translate.to_string();
            }
        }

        key.to_string()
    }

    pub fn variables(&self, lang: &str, key: &str, variables: Vec<TranslatorVariable>) -> String {
        self.apply_variables(self.translate(lang, key), variables)
    }

    pub fn choices(
        &self,
        lang: &str,
        key: &str,
        value: i64,
        variables: Option<Vec<TranslatorVariable>>,
    ) -> String {
        let mut result = self.translate(lang, key);
        let result_split: Vec<&str> = result.split("|").collect();
        let result_len = result_split.len();
        let u_value = if value < 0 { value * -1 } else { value };

        // TODO: Оформить по нормальному
        let choices = match lang {
            "ru" => choices_rule_ru(u_value, result_len),
            "en" => choices_rule_en(u_value, result_len),
            _ => 0,
        };
        if let Some(r) = result_split.get(choices) {
            result = r.to_string();
        } else {
            if let Some(r) = result_split.get(0) {
                result = r.to_string();
            }
        }

        if let Some(variables) = variables {
            self.apply_variables(result, variables)
        } else {
            result
        }
    }
}

pub enum TranslatorVariable {
    String(String, String),
    I32(String, i32),
    U64(String, u64),
    Usize(String, usize),
}

fn choices_rule_ru(value: i64, choices: usize) -> usize {
    let singular = value % 10 == 1 && value % 100 != 11;
    if choices == 2 {
        return if singular { 0 } else { 1 };
    }
    let few = value % 10 >= 2 && value % 10 <= 4 && (value % 100 < 10 || value % 100 >= 20);
    if singular {
        0
    } else {
        if few {
            1
        } else {
            2
        }
    }
}

fn choices_rule_en(value: i64, _: usize) -> usize {
    if value > 1 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn translate() {
        let config = Data::new(Config::new());
        let mut translates: HashMap<String, HashMap<String, String>> = HashMap::from([]);
        translates.insert(
            "en".to_string(),
            HashMap::from([
                ("test_key".to_string(), "test_value".to_string()),
                ("test_key2".to_string(), "test_value2".to_string()),
            ]),
        );
        let t: TranslatorService = TranslatorService::new(config, translates);

        assert_eq!("test_value".to_string(), t.translate("en", "test_key"));
        assert_eq!("test_value".to_string(), t.translate("fi", "test_key"));
        assert_eq!("test_key123".to_string(), t.translate("en", "test_key123"));
    }

    #[bench]
    fn bench_translate(b: &mut Bencher) {
        let config = Data::new(Config::new());
        let mut t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        t.insert(
            "en",
            "test_key".to_string(),
            "test_value :variable 213".to_string(),
        );
        t.insert("en", "test_key2".to_string(), "test_value2".to_string());

        b.iter(|| {
            t.translate("en", "test_key");
        });
    }

    #[test]
    fn variables() {
        let config = Data::new(Config::new());
        let mut t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        t.insert(
            "en",
            "test_key".to_string(),
            "test_value :variable 213".to_string(),
        );
        t.insert("en", "test_key2".to_string(), "test_value2".to_string());

        assert_eq!(
            "test_value test321 213".to_string(),
            t.variables(
                "en",
                "test_key",
                vec![TranslatorVariable::String(
                    "variable".to_string(),
                    "test321".to_string()
                ),]
            )
        );
    }

    #[bench]
    fn bench_variables(b: &mut Bencher) {
        // 170.20 ns/iter (+/- 5.15)
        let config = Data::new(Config::new());
        let mut t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        t.insert(
            "en",
            "test_key".to_string(),
            "test_value :variable 213".to_string(),
        );
        t.insert("en", "test_key2".to_string(), "test_value2".to_string());

        b.iter(|| {
            t.variables(
                "en",
                "test_key",
                vec![TranslatorVariable::String(
                    "variable".to_string(),
                    "test321".to_string(),
                )],
            );
        });
    }

    #[test]
    fn choices() {
        let config = Data::new(Config::new());
        let mut t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        t.insert("en", "test_key".to_string(), "second|seconds".to_string());
        t.insert(
            "ru",
            "test_key".to_string(),
            "секунда|секунды|секунд".to_string(),
        );

        let value = t.choices("en", "test_key", 1, None);
        assert_eq!("second".to_string(), value);
        let value = t.choices("en", "test_key", 2, None);
        assert_eq!("seconds".to_string(), value);
        let value = t.choices("ru", "test_key", 1, None);
        assert_eq!("секунда".to_string(), value);
        let value = t.choices("ru", "test_key", 2, None);
        assert_eq!("секунды".to_string(), value);
        let value = t.choices("ru", "test_key", 10, None);
        assert_eq!("секунд".to_string(), value);
        let value = t.choices("ru", "test_key", 100033, None);
        assert_eq!("секунды".to_string(), value);
    }

    #[bench]
    fn bench_format(b: &mut Bencher) {
        // 0.23 ns/iter (+/- 0.00)
        b.iter(|| {
            let _ = format!("test test test {key} {value}", key = "test", value = 123);
        });
    }

    #[bench]
    fn bench_replace(b: &mut Bencher) {
        // 139.26 ns/iter (+/- 5.05)
        let variables: HashMap<String, String> = HashMap::from([
            (":key".to_string(), "test123".to_string()),
            (":value".to_string(), "test321".to_string()),
        ]);

        b.iter(|| {
            let mut string = "test test test :key :value".to_string();
            for (key, value) in &variables {
                string = string.replace(key, value);
            }
        });
    }

    #[bench]
    fn bench_custom_format(b: &mut Bencher) {
        // 122.59 ns/iter (+/- 5.49)
        let variables: HashMap<String, String> = HashMap::from([
            (":key".to_string(), "test123".to_string()),
            (":value".to_string(), "test321".to_string()),
        ]);

        let s = "test test test :key :value".to_string();
        let chunks_: Vec<&str> = s.split(" ").collect();

        let mut chunks: Vec<String> = Vec::new();
        let mut chunk = "".to_string();
        for str in chunks_.iter() {
            let str = str.trim();
            if str.is_empty() {
                continue;
            }
            if let Some(char) = str.get(..1) {
                if char == ":" {
                    let c = chunk.trim().to_owned();
                    if !c.is_empty() {
                        chunks.push(c.to_string());
                    }
                    chunks.push(str.to_string());
                    chunk = "".to_string();
                } else {
                    chunk.push(' ');
                    chunk.push_str(str);
                }
            } else {
                chunk.push(' ');
                chunk.push_str(str);
            }
        }
        if !chunk.is_empty() {
            chunks.push(chunk.to_string());
        }
        b.iter(|| {
            let mut value = "".to_string();
            for chunk in &chunks {
                let s = variables.get(chunk).unwrap_or(chunk);
                value.push(' ');
                value.push_str(s);
            }
            value = value.trim().to_string();
        });
    }

    // #[bench]
    // fn bench_strfmt(b: &mut Bencher) {
    //     // 227.62 ns/iter (+/- 6.12)
    //     let vars: HashMap<String, String> = HashMap::from([
    //         ("key".to_string(), "test123".to_string()),
    //         ("value".to_string(), "test321".to_string())
    //     ]);
    //
    //     let fmt = "test test test {key} {value}".to_string();
    //     dbg!(strfmt::strfmt(&fmt, &vars).unwrap());
    //     b.iter(|| {
    //         let _ = strfmt::strfmt(&fmt, &vars).unwrap();
    //     });
    // }

    // #[bench]
    // fn bench_aho_corasick(b: &mut Bencher) {
    //     // 103.25 ns/iter (+/- 2.73)
    //     use aho_corasick::{AhoCorasick, MatchKind};
    //
    //     let patterns = &[":key", ":value"];
    //     let replace_with = &["test123", "test321"];
    //     let haystack = "test test test :key :value".to_string();
    //
    //     let ac = AhoCorasick::builder()
    //         .match_kind(MatchKind::LeftmostFirst)
    //         .build(patterns)
    //         .unwrap();
    //     let result = ac.replace_all(&haystack, replace_with);
    //     b.iter(|| {
    //         let _ = ac.replace_all(&haystack, replace_with);
    //     });
    // }

    // #[bench]
    // fn bench_formatify(b: &mut Bencher) {
    //     // 403.43 ns/iter (+/- 17.36)
    //     use formatify::PlaceholderFormatter;
    //     use formatify::Formatify;
    //
    //     let vars: HashMap<&str, String> = HashMap::from([
    //         ("key", "test123".to_string()),
    //         ("value", "test321".to_string())
    //     ]);
    //     let fmt = "test test test %(key) %(value)".to_string();
    //
    //     let formatter = Formatify::new();
    //     let formatted_string = formatter.replace_placeholders(&vars, &fmt);
    //     dbg!(formatted_string);
    //     b.iter(|| {
    //         let _ = formatter.replace_placeholders(&vars, &fmt);
    //     });
    // }

    #[test]
    fn new_from_files() {
        let config = Data::new(Config::new());
        let _ = TranslatorService::new_from_files(config).unwrap();
    }
}
