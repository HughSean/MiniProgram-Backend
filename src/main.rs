#![allow(non_snake_case)]
use axum::{http::StatusCode, Router};
use std::{future::IntoFuture, sync::Arc};
use tokio::sync;
use tracing::{info, warn};
mod api;
mod appstate;
mod cfg;
mod error;
mod module;
mod utils;
mod App {
    pub type Result<T> = anyhow::Result<T>;
}

#[tokio::main]
async fn main() {
    //设置日志
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_env_filter("backend=debug,axum=debug")
        // .with_max_level(Level::DEBUG)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::new(
            time::macros::format_description!("[year]-[month]-[day] ([hour]:[minute]:[second])"),
        ))
        .init();
    //配置

    let (sender, _) = sync::broadcast::channel(100);
    let cfg = cfg::parse().await.unwrap();
    let addrstr = format!("{}:{}", cfg.servercfg.ip, cfg.servercfg.port);
    let state = Arc::new(
        appstate::AppState::new(cfg, Arc::new(sender))
            .await
            .unwrap(),
    );
    //挂载路由
    let approuter = api::router(state.clone())
        .with_state(state.clone())
        .merge(api::test::router(state.clone()))
        .fallback(fallback);
    info!("路由挂载完成");
    //启动服务器
    info!("🚀👏 {}", addrstr);
    let serve_handler = tokio::spawn(
        axum::serve::serve(
            tokio::net::TcpListener::bind(addrstr).await.unwrap(),
            Router::new().nest("/api", approuter).into_make_service(),
        )
        .into_future(),
    );
    tokio::signal::ctrl_c().await.unwrap();
    serve_handler.abort();
    warn!("服务即将停止");
}

//失败路由
async fn fallback(uri: axum::http::Uri) -> (StatusCode, String) {
    warn!("fallback {}", uri);
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}

#[test]
fn test() {
    let t: Result<crate::module::order::OrdersOfCourt, _> =
        serde_json::from_slice(b"{\"court_id\":\"53fd6922-499e-46bb-a122-7ffcad2aa26c\"}");
    println!("{}", t.unwrap().court_id)
}
