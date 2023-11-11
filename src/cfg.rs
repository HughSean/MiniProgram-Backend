use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct Cfg {
    pub servercfg: ServerCfg,
    pub tokencfg: TokenCfg,
}

pub async fn parse() -> crate::App::Result<Cfg> {
    //读取文件
    let s = tokio::fs::read_to_string("./cfg.toml")
        .await
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    //解析配文件
    let cfg = toml::from_str::<Cfg>(&s).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    info!("cfg.toml 解析完成");
    Ok(cfg)
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerCfg {
    pub ip: String,
    pub port: u16,
    pub db_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenCfg {
    pub access_token_ttl: i64,
    pub access_prikey: String,
    pub access_pubkey: String,
}
