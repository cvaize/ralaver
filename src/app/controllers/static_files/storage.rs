use std::path::MAIN_SEPARATOR;
use std::sync::Arc;
use actix_web::web::{Data, Path, ReqData};
use actix_web::{error, Error, Result};
use actix_files::NamedFile;
use crate::{Config, Disk, FilePolicy, FileService, RoleService, User, UserFileService};

pub async fn public(
    config: Data<Config>,
    filename: Path<String>,
) -> Result<NamedFile, Error> {
    let mut path = config.filesystem.disks.local.public_root.to_owned();
    let filename = filename.into_inner();
    path.push(MAIN_SEPARATOR);
    path.push_str(&filename);

    Ok(NamedFile::open(path)?)
}

pub async fn private(
    user: ReqData<Arc<User>>,
    filename: Path<String>,
    role_service: Data<RoleService>,
    file_service: Data<FileService>,
) -> Result<NamedFile, Error> {
    let role_service = role_service.get_ref();
    let user = user.as_ref();

    let user_roles = role_service.get_all_throw_http()?;
    if !FilePolicy::can_show(&user, &user_roles) {
        return Err(error::ErrorForbidden(""));
    }

    let filename = filename.into_inner();
    let file_service = file_service.get_ref();
    let file = file_service.first_by_disk_and_filename_throw_http(&Disk::Local, &filename)?;

    Ok(NamedFile::open(file.path)?)
}
