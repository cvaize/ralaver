use std::path::MAIN_SEPARATOR;
use actix_web::web::{Data, Path};
use actix_web::{Error, Result};
use actix_files::NamedFile;
use crate::Config;

pub async fn invoke(
    config: Data<Config>,
    filename: Path<String>,
) -> Result<NamedFile, Error> {
    let mut path = config.filesystem.disks.local.public_root.to_owned();
    let filename = filename.into_inner();
    path.push(MAIN_SEPARATOR);
    path.push_str(&filename);

    // TODO: Добавить в публичный путь признак пользователя, так как метка публичности в user_file

    Ok(NamedFile::open(path)?)
}
