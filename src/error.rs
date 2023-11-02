use crate::cfg::CfgError;
// use crate::utils::auth::AuthError;

pub type Result<T> = anyhow::Result<T, Error>;
// core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CfgError(CfgError),
    DbError(String),
}
