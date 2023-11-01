#![allow(non_snake_case)]
mod api;
mod appstate;
mod cfg;
mod error;
mod module;
mod utils;
use std::sync::Arc;

use crate::api::{login, register};
use axum::{http::StatusCode, response::IntoResponse, Router, Server};
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
    //设置关机信号
    let (rx, cx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        rx.send(()).ok();
    });
    //获取配置文件
    let cfg = cfg::parse().await.unwrap();
    let addrstr = format!("{}:{}", cfg.servercfg.ip, cfg.servercfg.port);
    info!("监听地址: {}", addrstr);
    let state = appstate::AppState::new(cfg).await.unwrap();
    let state = Arc::new(state);
    //设置服务路由
    let approuter = Router::new()
        .nest("/api", login::router().merge(register::router()))
        .with_state(state.clone())
        .fallback(fallback);
    //启动服务器
    Server::bind(&addrstr.parse().unwrap())
        .serve(approuter.into_make_service())
        .with_graceful_shutdown(async move {
            cx.await.ok();
            warn!("收到关机信号，即将关机")
        })
        .await
        .unwrap();
}
//默认失败路由
async fn fallback() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

mod test {
    #[tokio::test]
    async fn t() {}
}
