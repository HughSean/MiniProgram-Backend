use pbkdf2::password_hash::{PasswordVerifier, SaltString};
use tracing::{error, info};
use uuid::Uuid;

use super::error::HandleErr;

//密码散列
pub fn hash_password<T>(pwd: &str) -> Result<String, HandleErr<T>> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let password = pwd.as_bytes();
    let password_hash =
        pbkdf2::password_hash::PasswordHasher::hash_password(&pbkdf2::Pbkdf2, password, &salt)
            .map_err(|err| {
                let id = Uuid::new_v4();
                error!("{} >>>> {}", id, err.to_string());
                HandleErr::ServerInnerErr(id)
            })?
            .to_string();
    info!("密码散列成功");
    Ok(password_hash)
}

//密码校验
pub fn verify_password(pwd: &str, password_hash: &str) -> Result<(), HandleErr<&'static str>> {
    let parsed_hash = pbkdf2::password_hash::PasswordHash::new(password_hash).map_err(|err| {
        let id = Uuid::new_v4();
        error!("{} >>>> {}", id, err.to_string());
        HandleErr::ServerInnerErr(id)
    })?;
    pbkdf2::Pbkdf2
        .verify_password(pwd.as_bytes(), &parsed_hash)
        .or(Err(HandleErr::BadRequest(-1, "密码校验错误")))?;
    info!("密码检验通过");
    Ok(())
}
