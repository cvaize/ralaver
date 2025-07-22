use crate::helpers::collect_files_from_dir;
use crate::{Config, KeyValueService};
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
            log::error!("TranslatorService::new_from_files - {e}");
            e
        })?;
        dir.push(Path::new(&config.get_ref().translator.translates_folder));
        let str_dir = dir.to_owned();
        let str_dir = str_dir.to_str().unwrap();

        let collect_paths: Vec<PathBuf> = collect_files_from_dir(dir.as_path()).map_err(|e| {
            log::error!("TranslatorService::new_from_files - {e}");
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
                log::error!("TranslatorService::new_from_files - {e}");
                e
            })?;

            let flat_json: String = flatten_json::flatten_from_str(&content).map_err(|e| {
                log::error!("TranslatorService::new_from_files - {e}");
                e
            })?;
            let flatten_keys: Value = serde_json::from_str(&flat_json).map_err(|e| {
                log::error!("TranslatorService::new_from_files - {e}");
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

        let mut translator_service = Self::new(config, save_translates);
        translator_service.compile_inner_variables();
        Ok(translator_service)
    }

    pub fn compile_inner_variables(&mut self) {
        for _ in 0..5 {
            let mut insert_rows: Vec<[String; 3]> = Vec::new();

            for (lang, translates) in &self.translates {
                for (key, value) in translates {
                    let mut variables: HashMap<String, String> = HashMap::new();
                    let start_strings: Vec<&str> = value.split("{{").collect();
                    for start_string in start_strings {
                        let end_strings: Vec<&str> = start_string.split("}}").collect();
                        if end_strings.len() > 1 {
                            variables.insert(
                                end_strings[0].trim().to_owned(),
                                end_strings[0].to_owned(),
                            );
                        }
                    }

                    if variables.len() > 0 {
                        let mut value_ = value.to_owned();

                        for (variable, variable_replace) in variables {
                            let variable_value: &str =
                                self.get(lang, &variable).to_owned().unwrap_or(&variable);
                            if variable_value.ne(&variable) {
                                let mut pattern: String = "{{".to_string();
                                pattern.push_str(&variable_replace);
                                pattern.push_str("}}");
                                value_ = value_.replace(&pattern, variable_value);
                            }
                        }

                        insert_rows.push([lang.to_owned(), key.to_owned(), value_]);
                    }
                }
            }

            for insert_row in &insert_rows {
                self.insert(
                    &insert_row[0],
                    insert_row[1].to_owned(),
                    insert_row[2].to_owned(),
                );
            }

            if insert_rows.len() == 0 {
                break;
            }
        }
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

    pub fn is(&self, lang: &str, key: &str) -> bool {
        if let Some(ts) = self.translates.get(lang) {
            return ts.contains_key(key);
        }
        false
    }

    fn v_key(&self, key: &str) -> String {
        let mut v_key = ":".to_string();
        v_key.push_str(key);
        v_key
    }

    pub fn translate(&self, lang: &str, key: &str) -> String {
        if let Some(translate) = self.get(lang, key) {
            return translate.to_string();
        }

        let app = &self.config.get_ref().app;

        if lang != app.fallback_locale && app.locale != app.fallback_locale {
            if let Some(translate) = self.get(&app.fallback_locale, key) {
                return translate.to_string();
            }
        }

        key.to_string()
    }

    pub fn contains(&self, lang: &str, key: &str) -> bool {
        if self.is(lang, key) {
            return true;
        }

        let app = &self.config.get_ref().app;
        if lang != app.locale {
            if self.is(&app.locale, key) {
                return true;
            }
        }

        if lang != app.fallback_locale && app.locale != app.fallback_locale {
            if self.is(&app.fallback_locale, key) {
                return true;
            }
        }

        false
    }

    pub fn choices(
        &self,
        lang: &str,
        key: &str,
        value: i64,
        vars: Option<&HashMap<&str, &str>>,
    ) -> String {
        let mut result = self.translate(lang, key);
        let result_split: Vec<&str> = result.split("|").collect();
        let result_len = result_split.len();
        let u_value = if value < 0 { value * -1 } else { value };

        // TODO: Добавить кеширование с ограничением на количество элементов в HashMap кеша
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

        if let Some(vars) = vars {
            self.apply_variables(result, vars)
        } else {
            result
        }
    }

    fn apply_variables(&self, mut value: String, vars: &HashMap<&str, &str>) -> String {
        for (k, v) in vars.iter() {
            value = value.replace(self.v_key(k).as_str(), v);
        }
        value
    }

    pub fn variables(&self, lang: &str, key: &str, vars: &HashMap<&str, &str>) -> String {
        self.apply_variables(self.translate(lang, key), vars)
    }
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

pub trait TranslatableError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use crate::preparation;

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

    #[bench]
    fn bench_variable(b: &mut Bencher) {
        // 123.49 ns/iter (+/- 4.11)
        let config = Data::new(Config::new());
        let mut t: TranslatorService = TranslatorService::new_from_files(config).unwrap();
        t.insert(
            "en",
            "test_key".to_string(),
            "test_value :variable 213".to_string(),
        );
        t.insert("en", "test_key2".to_string(), "test_value2".to_string());

        b.iter(|| {
            let mut vars = HashMap::new();
            vars.insert("variable", "test321");
            t.variables("en", "test_key", &vars);
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
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo bench -- --nocapture --exact app::services::translator::tests::bench_format
        // 31.64 ns/iter (+/- 1.11)
        let var1 = "test";
        let var2 = 123.to_string();
        let var2 = var2.as_str();
        b.iter(|| {
            let _ = format!("test test test {var1} {var2}");
        });
    }

    #[bench]
    fn bench_push_str(b: &mut Bencher) {
        // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo bench -- --nocapture --exact app::services::translator::tests::bench_push_str
        // 40.85 ns/iter (+/- 0.93)
        let var1 = "test";
        let var2 = 123.to_string();
        let var2 = var2.as_str();
        b.iter(|| {
            let mut str = "test test test ".to_string();
            str.push_str(var1);
            str.push_str(" ");
            str.push_str(var2);
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
            let _ = value.trim().to_string();
        });
    }

    #[test]
    fn new_from_files() {
        let config = Data::new(Config::new());
        let _ = TranslatorService::new_from_files(config).unwrap();
    }
    //
    // #[bench]
    // fn bench_hash_map_vs_redis(b: &mut Bencher) {
    // // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo bench -- --nocapture --exact app::services::translator::tests::bench_hash_map_vs_redis
    // // #[test]
    // // fn test_hash_map_vs_redis() {
    // //     // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::services::translator::tests::test_hash_map_vs_redis
    //     let (_, all_services) = preparation();
    //     let key_value_service = all_services.key_value_service.get_ref();
    //     let translator_service = all_services.translator_service.get_ref();
    //     // let mut last_key = "".to_string();
    //     // for (lang, translates) in &translator_service.translates {
    //     //     for (key, value) in translates {
    //     //         last_key = format!("{lang}.{key}");
    //     //         key_value_service.set_ex(&last_key, value, 86400).unwrap();
    //     //     }
    //     // }
    //     // dbg!(&last_key);
    //     let v: Option<String> = key_value_service.get("en.validation.password.symbols").unwrap();
    //     dbg!(&v);
    //     let v = translator_service.get("en", "validation.password.symbols");
    //     dbg!(&v);
    //     b.iter(|| {
    //         // 47,511.95 ns/iter (+/- 4,466.43)
    //         // let _: Option<String> = key_value_service.get("en.validation.password.symbols").unwrap();
    //         // 36.55 ns/iter (+/- 0.96)
    //         let _ = translator_service.get("en", "validation.password.symbols");
    //     });
    // }
    //
    // #[bench]
    // fn bench_kv_vs_redis(b: &mut Bencher) {
    //     // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo bench -- --nocapture --exact app::services::translator::tests::bench_kv_vs_redis
    // // #[test]
    // // fn test_kv_vs_redis() {
    // //     // RUSTFLAGS=-Awarnings CARGO_INCREMENTAL=0 cargo test -- --nocapture --exact app::services::translator::tests::test_kv_vs_redis
    //     use kv::*;
    //
    //     let (_, all_services) = preparation();
    //     let key_value_service = all_services.key_value_service.get_ref();
    //     let translator_service = all_services.translator_service.get_ref();
    //
    //     // При реализации expires можно при получении данных проверять expires и если он истёк, то удалять старое значение и отдавать в ответе None.
    //     let cfg = Config::new("./storage/kv_db");
    //     let store = Store::new(cfg).unwrap();
    //     let bucket = store.bucket::<Raw, Raw>(Some("bucket_name")).unwrap();
    //
    //     // for (lang, translates) in &translator_service.translates {
    //     //     for (key, value) in translates {
    //     //         let k = Raw::from(format!("{lang}.{key}").into_bytes());
    //     //         let v = Raw::from(value.to_owned().into_bytes());
    //     //         bucket.set(&k, &v).unwrap();
    //     //     }
    //     // }
    //     // "en.validation.password.symbols"
    //     let key = Raw::from(b"en.validation.password.symbols");
    //     let v = String::from_utf8(bucket.get(&key).unwrap().unwrap().to_vec()).unwrap();
    //     dbg!(&v);
    //     let v = translator_service.get("en", "validation.password.symbols");
    //     dbg!(&v);
    //     b.iter(|| {
    //         // 190.75 ns/iter (+/- 6.52)
    //         let _ = bucket.get(&key).unwrap().unwrap();
    //         // 47,511.95 ns/iter (+/- 4,466.43)
    //         // let _: Option<String> = key_value_service.get("en.validation.password.symbols").unwrap();
    //         // 36.55 ns/iter (+/- 0.96)
    //         // let _ = translator_service.get("en", "validation.password.symbols");
    //     });
    // }
}
