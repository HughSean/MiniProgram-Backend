#![allow(non_snake_case)]
mod api;
mod appstate;
mod cfg;
mod module;
mod utils;
use axum::{http::StatusCode, response::IntoResponse, Router, Server};
use std::sync::Arc;
use tracing::{info, warn};

mod App {
    pub type Result<T> = anyhow::Result<T>;
}

#[tokio::main]
async fn main() {
    //设置日志
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter("backend=debug")
        .init();
    //获取配置文件
    let cfg = cfg::parse().await.unwrap();
    let addrstr = format!("{}:{}", cfg.servercfg.ip, cfg.servercfg.port);
    let state = Arc::new(appstate::AppState::new(cfg).await.unwrap());
    //挂载路由
    let approuter = api::router(state.clone())
        .with_state(state.clone())
        .merge(api::test::router(state.clone()))
        .fallback(fallback);
    info!("路由挂载完成");
    //启动服务器
    info!("🚀👏 {}", addrstr);
    Server::bind(&addrstr.parse().unwrap())
        .serve(Router::new().nest("/api", approuter).into_make_service())
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
