use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug)]
pub enum HandleErr<T> {
    BadRequest(i32, T),
    ServerInnerErr(Uuid),
    UnAuthorized,
}

impl<T: serde::Serialize + ToString> IntoResponse for HandleErr<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            HandleErr::BadRequest(code, err) => {
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
            HandleErr::ServerInnerErr(id) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "code":-1,
                    "msg":format!("服务器内部错误, 错误代码{}", id.to_string()),
                })),
            )
                .into_response(),
            HandleErr::UnAuthorized => StatusCode::UNAUTHORIZED.into_response(),
        }
    }
}

macro_rules! impl_into {
    ($T:ty, $U:ty) => {
        impl Into<HandleErr<$T>> for HandleErr<$U> {
            fn into(self) -> HandleErr<$T> {
                match self {
                    HandleErr::BadRequest(code, msg) => HandleErr::BadRequest(code, msg.into()),
                    HandleErr::ServerInnerErr(id) => HandleErr::ServerInnerErr(id),
                    HandleErr::UnAuthorized => HandleErr::UnAuthorized,
                }
            }
        }
    };
}

impl_into!(String, &'static str);
