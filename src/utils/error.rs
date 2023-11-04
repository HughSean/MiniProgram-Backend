use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug)]
pub enum BaseError<T> {
    BadRequest(i8, T),
    ServerInnerErr,
}

impl<T: serde::Serialize> IntoResponse for BaseError<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            BaseError::BadRequest(code, err) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "code":code,
                    "msg":err
                })),
            )
                .into_response(),
            BaseError::ServerInnerErr => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code":-1,
                    "msg":"服务器内部错误"
                })),
            )
                .into_response(),
        }
    }
}
