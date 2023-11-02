use crate::appstate::AppState;
use axum::Json;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::debug;

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

#[derive(Debug, Serialize, Deserialize)]
enum TokenError {}

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
    debug!("token encoding");
    let token = jsonwebtoken::encode(
        &header,                                                           //头部
        &claims,                                                           //有效载荷
        &jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes())?, //签名
    )?;
    token_details.token = Some(token);
    Ok(token_details)
}

pub fn jwt_token_verify(
    token: &str,
    public_key: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    debug!("token verifing");
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    let decoded: jsonwebtoken::TokenData<TokenClaims> = jsonwebtoken::decode(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())?,
        &validation,
    )?;
    let user_id = uuid::Uuid::parse_str(decoded.claims.sub.as_str()).unwrap();
    Ok(TokenDetails {
        token: None,
        token_uuid: decoded.claims.token_uuid,
        user_id,
        expires_in: None,
    })
}

pub async fn save_token_data_to_redis(
    data: &AppState,
    token_details: &TokenDetails,
    ttl: i64,
) -> Result<(), Json<serde_json::Value>> {
    debug!("saving token to redis");
    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": -4,
                "msg": format!("Redis error: {}", e),
            });
            Json(error_response)
        })?;

    //设置<token_uuid, user_id>
    redis_client
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (ttl * 60) as usize,
        )
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "code": -4,
                "msg": format_args!("{}", e),
            });
            Json(error_response)
        })?;
    Ok(())
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn test1() {}
}
