use crate::error::{Error, Result};
use serde::Deserialize;
use tracing::info;

#[derive(Debug)]
pub enum CfgError {
    FileReadErr(String),
    TomlParseErr(String),
}
impl Into<Error> for CfgError {
    fn into(self) -> Error {
        Error::CfgError(self)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cfg {
    pub servercfg: ServerCfg,
    pub jwtcfg: JwtCfg,
    pub security: Security,
}

pub async fn parse() -> Result<Cfg> {
    //读取文件
    let s = tokio::fs::read_to_string("./cfg.toml")
        .await
        .map_err(|err| CfgError::FileReadErr(err.to_string()).into())?;
    //解析配文件
    let cfg =
        toml::from_str::<Cfg>(&s).map_err(|err| CfgError::TomlParseErr(err.to_string()).into())?;
    info!("cfg.toml 解析完成");
    Ok(cfg)
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerCfg {
    pub ip: String,
    pub port: u16,
    pub db_url: String,
    pub redis_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtCfg {
    pub jwt_secret: String,
    pub jwt_expires_in: i32,
    pub jwt_maxage: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Security {
    // pub salt_string: String,
    pub access_prikey: String,
    pub access_pubkey: String,
    pub refresh_prikey: String,
    pub refresh_pubkey: String,
}
