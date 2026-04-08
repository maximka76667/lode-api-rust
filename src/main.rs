use std::sync::Arc;

use lode_api_rust::{AppState, build_router};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lode_api_rust=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = SqlitePoolOptions::new()
        .connect("sqlite:./readings.db?mode=rwc")
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    let (tx, _) = broadcast::channel(32);
    let state = Arc::new(AppState { db, tx });
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3111").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
