mod handlers;
pub mod models;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
    response::IntoResponse,
};
use tokio::sync::broadcast;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub struct AppState {
    pub db: sqlx::PgPool,
    pub tx: broadcast::Sender<models::SensorReading>,
}

async fn index() -> impl IntoResponse {
    "lode-api-rust is running"
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/readings", post(handlers::create_reading))
        .route("/readings", get(handlers::get_readings))
        .route("/readings/latest", get(handlers::get_latest_reading))
        .route("/sse", get(handlers::sse_handler))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
