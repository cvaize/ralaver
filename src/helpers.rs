use std::{fs, io};
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}
#[allow(dead_code)]
pub fn dbg_type_of<T>(_: &T) {
    dbg!(std::any::type_name::<T>());
}

pub fn collect_files_from_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
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

#[macro_export]
macro_rules! log_map_err {
    ($error:expr, $message:expr) => {
|e| {
    log::error!("{}", format!("{} - {:}", $message, &e).as_str());
    return $error;
}
    };
}
