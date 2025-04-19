use crate::config::RedisDbConfig;
use r2d2::Pool;
use redis::{Client, ErrorKind, RedisError, Value};
use strum_macros::{Display, EnumString};

pub type RedisPool = Pool<Client>;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum RedisConnectionError {
    CreateClientFail,
    CreatePoolFail,
    GetSessionStoreFail,
}

pub fn get_connection_pool(
    config: &RedisDbConfig,
) -> Result<RedisPool, RedisConnectionError> {
    log::info!("{}","Connecting to Redis database.");
    let database_url = config.url.to_owned();

    let client = Client::open(database_url).map_err(|e| {
        log::error!("{}",format!("RedisConnectionError::CreateClientFail - {:}", &e).as_str());
        RedisConnectionError::CreateClientFail
    })?;

    Pool::builder().build(client).map_err(|e| {
        log::error!("{}",format!("RedisConnectionError::CreatePoolFail - {:}", &e).as_str());
        RedisConnectionError::CreatePoolFail
    })
}


pub fn get_inner_value(v: &Value) -> &Value {
    if let Value::Attribute {
        data,
        attributes: _,
    } = v
    {
        data.as_ref()
    } else {
        v
    }
}

pub fn get_owned_inner_value(v: Value) -> Value {
    if let Value::Attribute {
        data,
        attributes: _,
    } = v
    {
        *data
    } else {
        v
    }
}

pub fn make_redis_error(v: &Value, m: &str) -> RedisError {
    RedisError::from((
        ErrorKind::TypeError,
        "Response was of incompatible type",
        format!("{:?} (response was {:?})", m, v),
    ))
}

pub static REDIS_ERROR_MESSAGE: &str = "Response type not model compatible.";

#[macro_export]
macro_rules! model_redis_impl {
    ($t:ty) => {
impl redis::ToRedisArgs for $t {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(&serde_bare::to_vec(self).unwrap())
    }
}

impl redis::FromRedisValue for $t {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let v = crate::redis_connection::get_inner_value(v);
        match *v {
            redis::Value::BulkString(ref bytes) => serde_bare::from_slice(bytes).map_err(|_| {
                crate::redis_connection::make_redis_error(
                    v,
                    crate::redis_connection::REDIS_ERROR_MESSAGE,
                )
            }),
            _ => Err(crate::redis_connection::make_redis_error(
                v,
                crate::redis_connection::REDIS_ERROR_MESSAGE,
            )),
        }
    }
}
    };
}