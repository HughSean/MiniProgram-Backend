use std::sync::Arc;

use crate::{cfg::Cfg, utils::ws::Msg};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub cfg: Cfg,
    pub sender: Arc<tokio::sync::broadcast::Sender<Msg>>,
}

impl AppState {
    pub async fn new(
        cfg: Cfg,
        sender: Arc<tokio::sync::broadcast::Sender<Msg>>,
    ) -> crate::App::Result<Self> {
        Ok(Self {
            db: sea_orm::Database::connect(&cfg.servercfg.db_url)
                .await
                .unwrap(),
            cfg,
            sender,
        })
    }
}
