use axum::{http::StatusCode, BoxError};

async fn error_handler(err: BoxError) -> (StatusCode, String) {
    match err.downcast_ref::<crate::api::error::HandlerErr>().unwrap() {
        crate::api::error::HandlerErr::BadRequest(_) => todo!(),
        crate::api::error::HandlerErr::ServerInnerErr => todo!(),
    }
}
