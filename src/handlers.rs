use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{
        Json, Sse,
        sse::{Event, KeepAlive},
    },
};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::{
    AppState,
    models::{NewReading, ReadingFilters, SensorReading},
};

pub async fn create_reading(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewReading>,
) -> StatusCode {
    let result = sqlx::query_as::<_, SensorReading>(
        "INSERT INTO readings (temperature, humidity, pressure) \
         VALUES ($1, $2, $3) \
         RETURNING id, temperature, humidity, pressure, recorded_at",
    )
    .bind(body.temperature_c)
    .bind(body.humidity_pct)
    .bind(body.pressure_hpa)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(reading) => {
            let _ = state.tx.send(reading);
            StatusCode::CREATED
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn get_readings(
    State(state): State<Arc<AppState>>,
    Query(filters): Query<ReadingFilters>,
) -> Result<Json<Vec<SensorReading>>, StatusCode> {
    let mut qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        "SELECT id, temperature, humidity, pressure, recorded_at FROM readings WHERE 1=1",
    );

    if let Some(from) = filters.from {
        qb.push(" AND recorded_at >= ").push_bind(from);
    }
    if let Some(to) = filters.to {
        qb.push(" AND recorded_at <= ").push_bind(to);
    }

    qb.push(" ORDER BY recorded_at DESC");

    if let Some(limit) = filters.limit {
        qb.push(" LIMIT ").push_bind(limit);
    }

    qb.build_query_as::<SensorReading>()
        .fetch_all(&state.db)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_latest_reading(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SensorReading>, StatusCode> {
    sqlx::query_as::<_, SensorReading>(
        "SELECT id, temperature, humidity, pressure, recorded_at \
         FROM readings ORDER BY recorded_at DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)
    .map(Json)
}

pub async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.tx.subscribe()).filter_map(|result| {
        result.ok().and_then(|reading| {
            serde_json::to_string(&reading)
                .ok()
                .map(|json| Ok(Event::default().data(json)))
        })
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
