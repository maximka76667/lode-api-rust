use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use lode_api_rust::{AppState, build_router, models::SensorReading};
use tokio::sync::broadcast;
use tower::ServiceExt;

async fn setup() -> Arc<AppState> {
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    let (tx, _) = broadcast::channel(32);
    Arc::new(AppState { db, tx })
}

async fn post_reading(app: axum::Router, temp: f64, humidity: f64, pressure: f64) -> StatusCode {
    let body = serde_json::json!({
        "temperature_c": temp,
        "humidity_pct": humidity,
        "pressure_hpa": pressure,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/readings")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    response.status()
}

#[tokio::test]
async fn test_post_reading_returns_201() {
    let state = setup().await;
    let app = build_router(state);
    let status = post_reading(app, 23.4, 58.2, 1013.25).await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_readings_empty() {
    let state = setup().await;
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let readings: Vec<SensorReading> = serde_json::from_slice(&bytes).unwrap();
    assert!(readings.is_empty());
}

#[tokio::test]
async fn test_get_readings_returns_inserted() {
    let state = setup().await;

    post_reading(build_router(Arc::clone(&state)), 23.4, 58.2, 1013.25).await;
    post_reading(build_router(Arc::clone(&state)), 24.0, 60.0, 1012.0).await;

    let app = build_router(Arc::clone(&state));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/readings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let readings: Vec<SensorReading> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(readings.len(), 2);
}

#[tokio::test]
async fn test_get_readings_limit() {
    let state = setup().await;

    for i in 0..5 {
        post_reading(build_router(Arc::clone(&state)), i as f64, 50.0, 1000.0).await;
    }

    let app = build_router(Arc::clone(&state));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/readings?limit=3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let readings: Vec<SensorReading> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(readings.len(), 3);
}

#[tokio::test]
async fn test_post_invalid_body_returns_422() {
    let state = setup().await;
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/readings")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"bad": "data"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
