use serde::{Deserialize, Serialize};
use tracing::{info, warn};
// use redis::AsyncCommands;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,            //用户标识
    pub token_uuid: uuid::Uuid, //
    pub exp: i64,               //过期时间
    pub iat: i64,               //发布时间
    pub nbf: i64,               //生效时间
}

#[deprecated]
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenWrap {
    pub token: Option<String>,   //TokenClaims的JWT
    pub token_uuid: uuid::Uuid,  //token的uuid
    pub user_id: uuid::Uuid,     //token所属用户的uuid
    pub expires_in: Option<i64>, //过期时间戳
}

pub fn create(
    user_id: uuid::Uuid,
    ttl: i64,
    private_key: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        token_uuid: uuid::Uuid::new_v4(),
        exp: (now + chrono::Duration::minutes(ttl)).timestamp(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
    };
    //头部规定RS算法
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(
        &header,                                                           //头部
        &claims,                                                           //有效载荷
        &jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes())?, //签名
    )
    .map_err(|err| {
        warn!("token生成错误: {}", err.to_string());
        err
    })?;
    info!("token生成成功");
    Ok(token)
}

pub fn verify(token: &str, public_key: &str) -> Result<uuid::Uuid, jsonwebtoken::errors::Error> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    let decoded: jsonwebtoken::TokenData<TokenClaims> = jsonwebtoken::decode(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())?,
        &validation,
    )?;
    let user_id = uuid::Uuid::parse_str(decoded.claims.sub.as_str()).unwrap();
    info!("token检验通过");
    Ok(user_id)
}
