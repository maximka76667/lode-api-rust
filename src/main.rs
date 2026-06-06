use std::sync::Arc;
use std::time::Duration;

use lode_api_rust::models::TimestampedReading;
use lode_api_rust::{AppState, build_router, spawn_buffer_task};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::{broadcast, mpsc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lode_api_rust=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    let (tx, _) = broadcast::channel::<TimestampedReading>(32);
    let (buffer_tx, buffer_rx) = mpsc::channel(1024);

    let state = Arc::new(AppState { db: db.clone(), tx, buffer_tx });
    spawn_buffer_task(db, buffer_rx, Duration::from_secs(30));

    let app = build_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3600".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
