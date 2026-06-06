mod handlers;
pub mod models;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    response::IntoResponse,
    routing::{get, post},
};
use tokio::sync::{broadcast, mpsc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub struct AppState {
    pub db: sqlx::PgPool,
    pub tx: broadcast::Sender<models::TimestampedReading>,
    pub buffer_tx: mpsc::Sender<models::TimestampedReading>,
}

pub fn spawn_buffer_task(
    db: sqlx::PgPool,
    mut buffer_rx: mpsc::Receiver<models::TimestampedReading>,
    flush_interval: Duration,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(flush_interval);
        interval.tick().await; // skip the immediate first tick
        let mut buffer = Vec::new();

        loop {
            tokio::select! {
                reading = buffer_rx.recv() => {
                    match reading {
                        Some(r) => buffer.push(r),
                        None => break,
                    }
                }
                _ = interval.tick() => {
                    if buffer.is_empty() {
                        continue;
                    }
                    let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(
                        "INSERT INTO readings \
                         (temperature, humidity, pressure, \
                          presence_status, movement_distance_cm, movement_energy, \
                          stationary_distance_cm, stationary_energy, detection_distance_cm, \
                          recorded_at) "
                    );
                    qb.push_values(buffer.drain(..), |mut b, r: models::TimestampedReading| {
                        b.push_bind(r.data.temperature)
                         .push_bind(r.data.humidity)
                         .push_bind(r.data.pressure)
                         .push_bind(r.data.presence_status)
                         .push_bind(r.data.movement_distance_cm)
                         .push_bind(r.data.movement_energy)
                         .push_bind(r.data.stationary_distance_cm)
                         .push_bind(r.data.stationary_energy)
                         .push_bind(r.data.detection_distance_cm)
                         .push_bind(r.recorded_at);
                    });
                    if let Err(e) = qb.build().execute(&db).await {
                        tracing::error!("batch insert failed: {e}");
                    }
                }
            }
        }
    });
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
