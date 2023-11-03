#![allow(non_snake_case)]
mod api;
mod appstate;
mod cfg;
mod error;
mod module;
mod utils;
use axum::middleware;
use axum::{http::StatusCode, response::IntoResponse, Router, Server};
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    //设置日志
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or("backend=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
    //获取配置文件
    let cfg = cfg::parse().await.unwrap();
    let addrstr = format!("{}:{}", cfg.servercfg.ip, cfg.servercfg.port);
    let state = Arc::new(appstate::AppState::new(cfg).await.unwrap());
    //设置服务路由
    let protected_api =
    Router::new()
    .merge(api::admin::router())
    .layer(middleware::from_fn_with_state(
        state.clone(),
        utils::auth::auth,
    ));
    let api = Router::new()
        .merge(api::login::router())
        .merge(api::register::router());
    let approuter = Router::new()
        .nest("/api", api)
        .nest("/protected_api", protected_api)
        .with_state(state.clone())
        .merge(api::pub_test::router(state.clone()))
        .fallback(fallback);
    //启动服务器
    info!("监听地址: {}", addrstr);
    Server::bind(&addrstr.parse().unwrap())
        .serve(approuter.into_make_service())
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.unwrap();
            warn!("收到关机信号，即将关机")
        })
        .await
        .unwrap();
}
//默认失败路由
async fn fallback() -> impl IntoResponse {
    (StatusCode::BAD_GATEWAY, "not found")
}

mod test {
    #[tokio::test]
    async fn t() {}
}
