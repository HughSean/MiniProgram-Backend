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

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,   //TokenClaims的JWT
    pub token_uuid: uuid::Uuid,  //token的uuid
    pub user_id: uuid::Uuid,     //token所属用户的uuid
    pub expires_in: Option<i64>, //过期时间戳
}

pub fn jwt_token_gen(
    user_id: uuid::Uuid,
    ttl: i64,
    private_key: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let mut token_details = TokenDetails {
        token: None,
        token_uuid: uuid::Uuid::new_v4(),
        user_id,
        expires_in: Some((now + chrono::Duration::minutes(ttl)).timestamp()),
    };
    let claims = TokenClaims {
        sub: token_details.user_id.to_string(),
        token_uuid: token_details.token_uuid,
        exp: token_details.expires_in.unwrap(),
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
    token_details.token = Some(token);
    info!("token生成成功");
    Ok(token_details)
}

pub fn jwt_token_verify(
    token: &str,
    public_key: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    let decoded: jsonwebtoken::TokenData<TokenClaims> = jsonwebtoken::decode(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())?,
        &validation,
    )?;
    let user_id = uuid::Uuid::parse_str(decoded.claims.sub.as_str()).unwrap();
    info!("token检验通过");
    Ok(TokenDetails {
        token: None,
        token_uuid: decoded.claims.token_uuid,
        user_id,
        expires_in: None,
    })
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn test1() {}
}
