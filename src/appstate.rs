use crate::{cfg::Cfg, error::Error};
use sqlx::{Pool, Postgres};
use tracing::info;

type DB = Pool<Postgres>;
#[derive(Clone, Debug)]
pub struct AppState {
    pub redis_client: redis::Client,
    pub pgpool: DB,
    pub cfg: Cfg,
}

impl AppState {
    pub async fn new(cfg: Cfg) -> crate::error::Result<Self> {
        let redis_url = cfg.servercfg.redis_url.clone();
        // let salt_string = cfg.security.salt_string.clone();

        Ok(Self {
            redis_client: redis::Client::open(redis_url)
                .map_err(|err| Error::DbError(err.to_string()))?,
            pgpool: sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&cfg.servercfg.db_url)
                .await
                .map_err(|err| Error::DbError(err.to_string()))?,
            cfg,
        })
        .and_then(|s| {
            info!("程序状态创建完成");
            Ok(s)
        })
    }
}
