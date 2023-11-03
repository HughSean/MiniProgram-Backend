use crate::cfg::CfgError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CfgError(CfgError),
    DbError(String),
}
