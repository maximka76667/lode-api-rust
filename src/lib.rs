mod handlers;
pub mod models;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::broadcast;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub tx: broadcast::Sender<models::SensorReading>,
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/readings", post(handlers::create_reading))
        .route("/readings", get(handlers::get_readings))
        .route("/sse", get(handlers::sse_handler))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
