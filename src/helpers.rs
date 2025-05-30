use std::{fs, io};
use std::path::{Path, PathBuf};
use std::convert::TryInto;

#[allow(dead_code)]
pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}
#[allow(dead_code)]
pub fn dbg_type_of<T>(_: &T) {
    dbg!(std::any::type_name::<T>());
}

pub fn none_if_empty(v: &Option<String>) -> Option<String> {
    if let Some(v_) = v {
        let v = v_.trim();
        if v.len() != 0 {
            return Some(v.to_owned());
        }
    }
    None
}

pub fn vec_into_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn collect_files_from_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut result: Vec<PathBuf> = Vec::new();
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

pub fn get_sys_gettime_nsec() -> i64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC_COARSE, &mut time) };
    time.tv_nsec
}

#[allow(dead_code)]
pub fn dot_to_end(mut string: String) -> String {
    if string.ends_with('.') == false {
        string.push('.');
    }
    string
}

#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};
    use serde_json::json;
    use test::Bencher;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TestData {
        pub title: String,
        pub locale: String,
        pub locales: String,
        pub user: String,
        pub alerts: String,
        pub dark_mode: String,
        pub csrf: String,
        pub heading: String,
        pub main_str: String,
        pub extended_str: String,
        pub panel_str: String,
        pub users_str: String,
        pub create_str: String,
        pub form_action: String,
        pub form_method: String,
        pub form_fields_email_label: String,
        pub form_fields_email_value: String,
        pub form_fields_email_errors: String,
        pub form_fields_password_label: String,
        pub form_fields_password_value: String,
        pub form_fields_password_errors: String,
        pub form_fields_confirm_password_label: String,
        pub form_fields_confirm_password_value: String,
        pub form_fields_confirm_password_errors: String,
        pub form_fields_surname_label: String,
        pub form_fields_surname_value: String,
        pub form_fields_surname_errors: String,
        pub form_fields_name_label: String,
        pub form_fields_name_value: String,
        pub form_fields_name_errors: String,
        pub form_fields_patronymic_label: String,
        pub form_fields_patronymic_value: String,
        pub form_fields_patronymic_errors: String,
        pub form_fields_locale_label: String,
        pub form_fields_locale_value: String,
        pub form_fields_locale_errors: String,
        pub form_submit_label: String,
        pub form_errors: String,
    }

    #[bench]
    fn bench_str_sys_gettime_unsafe(b: &mut Bencher) {
        b.iter(|| {
            let mut time = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC_COARSE, &mut time) };
        });
    }

    #[bench]
    fn bench_str_sys_gettime_by_standard(b: &mut Bencher) {
        b.iter(|| std::time::SystemTime::now());
    }

    #[bench]
    fn json1(b: &mut Bencher) {
        // 2,616.94 ns/iter (+/- 360.94)
        b.iter(|| {
            json!({
                "title": "title",
                "locale": "locale",
                "locales": "locales",
                "user" : "user",
                "alerts": "req.get_alerts(&translator_service, &lang)",
                "dark_mode": "dark_mode",
                "csrf": "csrf",
                "heading": "heading",
                "main_str": "translator_service",
                "extended_str": "translator_service",
                "panel_str": "translator_service",
                "users_str": "translator_service",
                "create_str": "translator_service",
                "form": {
                    "action": "/users",
                    "method": "post",
                    "fields": {
                        "email": {
                            "label": "email_str",
                            "value": "&data.email",
                            "errors": "email_errors",
                        },
                        "password": {
                            "label": "password_str",
                            "value": "&data.password",
                            "errors": "password_errors",
                        },
                        "confirm_password": {
                            "label": "confirm_password_str",
                            "value": "&data.confirm_password",
                            "errors": "confirm_password_errors",
                        },
                        "surname": {
                            "label": "surname_str",
                            "value": "&data.surname",
                            "errors": "surname_errors",
                        },
                        "name": {
                            "label": "name_str",
                            "value": "&data.name",
                            "errors": "name_errors",
                        },
                        "patronymic": {
                            "label": "patronymic_str",
                            "value": "&data.patronymic",
                            "errors": "patronymic_errors",
                        },
                        "locale": {
                            "label": "locale_str",
                            "value": "&data.locale",
                            "errors": "locale_errors",
                        }
                    },
                    "submit": {
                        "label": "translator_service",
                    },
                    "errors": "form_errors"
                },
            });
        });
    }

    #[bench]
    fn json2(b: &mut Bencher) {
        // 2,635.05 ns/iter (+/- 295.70)
        b.iter(|| {
            json!({
                "title": "title",
                "locale": "locale",
                "locales": "locales",
                "user" : "user",
                "alerts": "req.get_alerts(&translator_service, &lang)",
                "dark_mode": "dark_mode",
                "csrf": "csrf",
                "heading": "heading",
                "main_str": "translator_service",
                "extended_str": "translator_service",
                "panel_str": "translator_service",
                "users_str": "translator_service",
                "create_str": "translator_service",
                "form_action": "/users",
                "form_method": "post",
                "form_fields_email_label": "email_str",
                "form_fields_email_value": "&data.email",
                "form_fields_email_errors": "email_errors",
                "form_fields_password_label": "password_str",
                "form_fields_password_value": "&data.password",
                "form_fields_password_errors": "password_errors",
                "form_fields_confirm_password_label": "confirm_password_str",
                "form_fields_confirm_password_value": "&data.confirm_password",
                "form_fields_confirm_password_errors": "confirm_password_errors",
                "form_fields_surname_label": "surname_str",
                "form_fields_surname_value": "&data.surname",
                "form_fields_surname_errors": "surname_errors",
                "form_fields_name_label": "name_str",
                "form_fields_name_value": "&data.name",
                "form_fields_name_errors": "name_errors",
                "form_fields_patronymic_label": "patronymic_str",
                "form_fields_patronymic_value": "&data.patronymic",
                "form_fields_patronymic_errors": "patronymic_errors",
                "form_fields_locale_label": "locale_str",
                "form_fields_locale_value": "&data.locale",
                "form_fields_locale_errors": "locale_errors",
                "form_submit_label": "translator_service",
                "form_errors": "form_errors",
            });
        });
    }

    #[bench]
    fn json3(b: &mut Bencher) {
        // 3,078.68 ns/iter (+/- 324.71)
        b.iter(|| {
            let data = TestData {
                title: "title".to_string(),
                locale: "locale".to_string(),
                locales: "locales".to_string(),
                user: "user".to_string(),
                alerts: "req.get_alerts(&translator_service, &lang)".to_string(),
                dark_mode: "dark_mode".to_string(),
                csrf: "csrf".to_string(),
                heading: "heading".to_string(),
                main_str: "translator_service".to_string(),
                extended_str: "translator_service".to_string(),
                panel_str: "translator_service".to_string(),
                users_str: "translator_service".to_string(),
                create_str: "translator_service".to_string(),
                form_action: "/users".to_string(),
                form_method: "post".to_string(),
                form_fields_email_label: "email_str".to_string(),
                form_fields_email_value: "&data.email".to_string(),
                form_fields_email_errors: "email_errors".to_string(),
                form_fields_password_label: "password_str".to_string(),
                form_fields_password_value: "&data.password".to_string(),
                form_fields_password_errors: "password_errors".to_string(),
                form_fields_confirm_password_label: "confirm_password_str".to_string(),
                form_fields_confirm_password_value: "&data.confirm_password".to_string(),
                form_fields_confirm_password_errors: "confirm_password_errors".to_string(),
                form_fields_surname_label: "surname_str".to_string(),
                form_fields_surname_value: "&data.surname".to_string(),
                form_fields_surname_errors: "surname_errors".to_string(),
                form_fields_name_label: "name_str".to_string(),
                form_fields_name_value: "&data.name".to_string(),
                form_fields_name_errors: "name_errors".to_string(),
                form_fields_patronymic_label: "patronymic_str".to_string(),
                form_fields_patronymic_value: "&data.patronymic".to_string(),
                form_fields_patronymic_errors: "patronymic_errors".to_string(),
                form_fields_locale_label: "locale_str".to_string(),
                form_fields_locale_value: "&data.locale".to_string(),
                form_fields_locale_errors: "locale_errors".to_string(),
                form_submit_label: "translator_service".to_string(),
                form_errors: "form_errors".to_string(),
            };
            serde_json::to_value(&data).unwrap();
        });
    }
}