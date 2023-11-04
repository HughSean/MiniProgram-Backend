use crate::cfg::Cfg;
use sqlx::{Pool, Postgres};
use tracing::info;

type DB = Pool<Postgres>;
#[derive(Clone, Debug)]
pub struct AppState {
    // pub redis_client: redis::Client,
    pub pgpool: DB,
    pub cfg: Cfg,
}

impl AppState {
    pub async fn new(cfg: Cfg) -> crate::App::Result<Self> {
        Ok(Self {
            pgpool: sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&cfg.servercfg.db_url)
                .await
                .and_then(|ok| {
                    info!("建立起和数据库的连接池");
                    Ok(ok)
                })
                .map_err(|err| anyhow::anyhow!(err.to_string()))?,
            cfg,
        })
    }
}
