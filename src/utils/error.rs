use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug)]
pub enum BaseError<T> {
    BadRequest(i8, T),
    ServerInnerErr(Uuid),
}

impl<T: serde::Serialize> IntoResponse for BaseError<T> {
    fn into_response(self) -> axum::response::Response {
        warn!("操作失败");
        match self {
            BaseError::BadRequest(code, err) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "code":code,
                    "msg":err
                })),
            )
                .into_response(),
            BaseError::ServerInnerErr(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code":-1,
                    "msg":"服务器内部错误",
                    "error_code":id
                })),
            )
                .into_response(),
        }
    }
}

impl From<BaseError<&'static str>> for BaseError<String> {
    fn from(value: BaseError<&'static str>) -> Self {
        match value {
            BaseError::BadRequest(code, msg) => Self::BadRequest(code, msg.into()),
            BaseError::ServerInnerErr(id) => Self::ServerInnerErr(id),
        }
    }
}
