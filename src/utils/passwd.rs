use pbkdf2::password_hash::{PasswordVerifier, SaltString};
//密码散列
pub fn passwd_encode(pwd: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut rand_core::OsRng);
    let password = pwd.as_bytes();
    let password_hash =
        pbkdf2::password_hash::PasswordHasher::hash_password(&pbkdf2::Pbkdf2, password, &salt)
            .map_err(|e| e.to_string())?
            .to_string();
    Ok(password_hash)
}
//密码校验
pub fn passwd_verify(pwd: &str, password_hash: &str) -> Result<(), String> {
    let parsed_hash =
        pbkdf2::password_hash::PasswordHash::new(password_hash).map_err(|err| err.to_string())?;

    pbkdf2::Pbkdf2
        .verify_password(pwd.as_bytes(), &parsed_hash)
        .map_err(|err| err.to_string())
}
