
use crate::{option_take_json_from_mysql_row, option_to_json_string_for_mysql, take_from_mysql_row, AppError, FromMysqlDto, MysqlColumnEnum, MysqlIdColumn, MysqlPool, MysqlQueryBuilder, MysqlRepository, PaginateParams, Role, RoleColumn, RoleServiceError, ToMysqlDto, UserFilter};
use actix_web::web::Data;
use mysql::Row;
use mysql::Value;
use strum_macros::{Display, EnumIter, EnumString};

pub struct RoleMysqlRepository {
    db_pool: Data<MysqlPool>,
}

impl MysqlRepository<Role, RolePaginateParams, RoleColumn, RoleFilter, RoleSort>
    for RoleMysqlRepository
{
    fn get_repository_name(&self) -> &str {
        "RoleMysqlRepository"
    }
    fn get_table(&self) -> &str {
        "roles"
    }
    fn get_db_pool(&self) -> &MysqlPool {
        self.db_pool.get_ref()
    }
}

impl RoleMysqlRepository {
    pub fn new(db_pool: Data<MysqlPool>) -> Self {
        Self { db_pool }
    }

    pub fn first_by_id(&self, id: u64) -> Result<Option<Role>, AppError> {
        let filters = vec![RoleFilter::Id(id)];
        self.first(&filters)
    }

    pub fn delete_by_id(&self, id: u64) -> Result<(), AppError> {
        let filters = vec![RoleFilter::Id(id)];
        self.delete(&filters)
    }

    pub fn delete_by_ids(&self, ids: &Vec<u64>) -> Result<(), AppError> {
        let filters = vec![RoleFilter::Ids(ids.to_owned())];
        self.delete(&filters)
    }

    pub fn first_by_code(&self, code: &str) -> Result<Option<Role>, AppError> {
        let filters: Vec<RoleFilter> = vec![RoleFilter::Code(code.to_string())];
        self.first(&filters)
    }

    pub fn exists_by_code(&self, code: &str) -> Result<bool, AppError> {
        let filters: Vec<RoleFilter> = vec![RoleFilter::Code(code.to_string())];
        self.exists(&filters)
    }

    pub fn delete_by_code(&self, code: &str) -> Result<(), AppError> {
        let filters: Vec<RoleFilter> = vec![RoleFilter::Code(code.to_string())];
        self.delete(&filters)
    }
}

pub type RolePaginateParams = PaginateParams<RoleFilter, RoleSort>;

#[derive(Debug)]
pub enum RoleFilter {
    Id(u64),
    Ids(Vec<u64>),
    Code(String),
    Search(String),
}

impl MysqlQueryBuilder for RoleFilter {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::Id(_) => query.push_str("id=:id"),
            Self::Ids(value) => {
                let mut v = "id in (".to_string();
                let ids: Vec<String> = value.iter().map(|d| d.to_string()).collect();
                let ids: String = ids.join(",").to_string();
                v.push_str(&ids);
                v.push_str(")");
                query.push_str(&v)
            },
            Self::Code(_) => query.push_str("code=:code"),
            Self::Search(_) => query.push_str("(name LIKE :search OR code LIKE :search)"),
        }
    }

    fn push_params_to_vec(&self, params: &mut Vec<(String, Value)>) {
        match self {
            Self::Id(value) => {
                params.push(("id".to_string(), Value::from(value)));
            }
            Self::Ids(_) => {}
            Self::Code(value) => {
                params.push(("code".to_string(), Value::from(value.to_string())));
            }
            Self::Search(value) => {
                let mut s = "%".to_string();
                s.push_str(value);
                s.push_str("%");
                params.push(("search".to_string(), Value::from(s)));
            }
        }
    }
}

#[derive(Debug, Display, EnumString, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum RoleSort {
    IdAsc,
    IdDesc,
    NameAsc,
    NameDesc,
    CodeAsc,
    CodeDesc,
}

impl MysqlQueryBuilder for RoleSort {
    fn push_params_to_mysql_query(&self, query: &mut String) {
        match self {
            Self::IdAsc => query.push_str("id ASC"),
            Self::IdDesc => query.push_str("id DESC"),
            Self::NameAsc => query.push_str("name ASC"),
            Self::NameDesc => query.push_str("name DESC"),
            Self::CodeAsc => query.push_str("code ASC"),
            Self::CodeDesc => query.push_str("code DESC"),
        };
    }

    fn push_params_to_vec(&self, _: &mut Vec<(String, Value)>) {}
}

impl ToMysqlDto<RoleColumn> for Role {
    fn push_mysql_param_to_vec(&self, column: &RoleColumn, params: &mut Vec<(String, Value)>) {
        match column {
            RoleColumn::Id => params.push((column.to_string(), Value::from(self.id.to_owned()))),
            RoleColumn::Code => {
                params.push((column.to_string(), Value::from(self.code.to_owned())))
            }
            RoleColumn::Name => {
                params.push((column.to_string(), Value::from(self.name.to_owned())))
            }
            RoleColumn::Description => {
                params.push((column.to_string(), Value::from(self.description.to_owned())))
            }
            RoleColumn::Permissions => {
                let permissions: Option<String> =
                    option_to_json_string_for_mysql(&self.permissions);
                params.push((column.to_string(), Value::from(permissions)))
            }
        }
    }
    fn get_id(&self) -> u64 {
        self.id
    }
}

impl FromMysqlDto for Role {
    fn take_from_mysql_row(row: &mut Row) -> Result<Self, AppError> {
        Ok(Self {
            id: take_from_mysql_row(row, RoleColumn::Id.to_string().as_str())?,
            name: take_from_mysql_row(row, RoleColumn::Name.to_string().as_str())?,
            code: take_from_mysql_row(row, RoleColumn::Code.to_string().as_str())?,
            description: take_from_mysql_row(row, RoleColumn::Description.to_string().as_str())?,
            permissions: option_take_json_from_mysql_row(
                row,
                RoleColumn::Permissions.to_string().as_str(),
            ),
        })
    }
}

impl MysqlColumnEnum for RoleColumn {}
impl MysqlIdColumn for RoleColumn {
    fn get_mysql_id_column() -> Self {
        Self::Id
    }
}
