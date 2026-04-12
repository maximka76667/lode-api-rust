use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use lode_api_rust::{AppState, build_router, models::SensorReading};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use tower::ServiceExt;

async fn setup() -> Arc<AppState> {
    dotenvy::dotenv().ok();
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    sqlx::query("TRUNCATE TABLE readings RESTART IDENTITY")
        .execute(&pool)
        .await
        .unwrap();

    let (tx, _) = broadcast::channel(32);
    Arc::new(AppState { db: pool, tx })
}

async fn post_reading(app: axum::Router, temp: f64, humidity: f64, pressure: f64) -> StatusCode {
    let body = serde_json::json!({
        "temperature_c": temp,
        "humidity_pct": humidity,
        "pressure_hpa": pressure,
    });

    app.oneshot(
        Request::builder()
            .method("POST")
            .uri("/readings")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
    .status()
}

#[tokio::test]
async fn test_post_reading_returns_201() {
    let state = setup().await;
    let status = post_reading(build_router(state), 23.4, 58.2, 1013.25).await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_get_readings_empty() {
    let state = setup().await;

    let response = build_router(state)
        .oneshot(Request::builder().uri("/readings").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let readings: Vec<lode_api_rust::models::SensorReading> =
        serde_json::from_slice(&bytes).unwrap();
    assert!(readings.is_empty());
}

#[tokio::test]
async fn test_get_readings_returns_inserted() {
    let state = setup().await;

    let s1 = post_reading(build_router(Arc::clone(&state)), 23.4, 58.2, 1013.25).await;
    let s2 = post_reading(build_router(Arc::clone(&state)), 24.0, 60.0, 1012.0).await;
    assert_eq!(s1, StatusCode::CREATED);
    assert_eq!(s2, StatusCode::CREATED);

    let response = build_router(Arc::clone(&state))
        .oneshot(Request::builder().uri("/readings").body(Body::empty()).unwrap())
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
        let status =
            post_reading(build_router(Arc::clone(&state)), i as f64, 50.0, 1000.0).await;
        assert_eq!(status, StatusCode::CREATED);
    }

    let response = build_router(Arc::clone(&state))
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

    let response = build_router(state)
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
