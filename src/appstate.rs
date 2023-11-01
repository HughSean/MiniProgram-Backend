use crate::{cfg::Cfg, error::Error};
use sqlx::{Pool, Postgres};
use tracing::info;

type DB = Pool<Postgres>;
#[derive(Clone, Debug)]
pub struct AppState {
    pub pgpool: DB,
    pub cfg: Cfg,
    pub redis_client: redis::Client,
    // pub salt: pbkdf2::password_hash::SaltString,
}

impl AppState {
    pub async fn new(cfg: Cfg) -> crate::error::Result<Self> {
        let redis_url = cfg.servercfg.redis_url.clone();
        // let salt_string = cfg.security.salt_string.clone();

        Ok(Self {
            pgpool: sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&cfg.servercfg.db_url)
                .await
                .map_err(|err| Error::DbError(err.to_string()))?,
            cfg,
            redis_client: redis::Client::open(redis_url)
                .map_err(|err| Error::DbError(err.to_string()))?,
            // salt: pbkdf2::password_hash::SaltString::from_b64(&salt_string)
            //     .map_err(|e| Error::SaltParseErr(e.to_string()))?,
        })
        .and_then(|s| {
            info!("程序状态创建完成");
            Ok(s)
        })
    }
}
