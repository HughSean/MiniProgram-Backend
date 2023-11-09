use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug)]
pub enum BaseError<T> {
    BadRequest(i32, T),
    ServerInnerErr(Uuid),
}

impl<T: serde::Serialize + ToString> IntoResponse for BaseError<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            BaseError::BadRequest(code, err) => {
                warn!("操作失败: {}", err.to_string());
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "code":code,
                        "msg":err
                    })),
                )
                    .into_response()
            }
            BaseError::ServerInnerErr(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code":-1,
                    "msg":format!("服务器内部错误, 错误代码{}", id.to_string()),
                })),
            )
                .into_response(),
        }
    }
}

macro_rules! impl_from {
    ($T:ty, $U:ty) => {
        impl From<BaseError<$T>> for BaseError<$U> {
            fn from(value: BaseError<$T>) -> Self {
                match value {
                    BaseError::BadRequest(code, msg) => Self::BadRequest(code, msg.into()),
                    BaseError::ServerInnerErr(id) => Self::ServerInnerErr(id),
                }
            }
        }
    };
}

impl_from!(&'static str, String);
