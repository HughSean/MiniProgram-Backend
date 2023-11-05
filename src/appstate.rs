use crate::cfg::Cfg;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub cfg: Cfg,
}

impl AppState {
    pub async fn new(cfg: Cfg) -> crate::App::Result<Self> {
        Ok(Self {
            db: sea_orm::Database::connect(&cfg.servercfg.db_url)
                .await
                .unwrap(),
            cfg,
        })
    }
}
