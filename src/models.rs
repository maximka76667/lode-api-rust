use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SensorReading {
    pub id: i64,
    #[serde(rename = "temperature_c")]
    pub temperature: f64,
    #[serde(rename = "humidity_pct")]
    pub humidity: f64,
    #[serde(rename = "pressure_hpa")]
    pub pressure: f64,
    pub recorded_at: DateTime<Utc>,
    pub presence_status: Option<i16>,
    pub movement_distance_cm: Option<i32>,
    pub movement_energy: Option<i16>,
    pub stationary_distance_cm: Option<i32>,
    pub stationary_energy: Option<i16>,
    pub detection_distance_cm: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct NewReading {
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub pressure_hpa: f64,
    // For human presence radar options are safeguards for first reading that could be None
    pub presence_status: Option<i16>,
    pub movement_distance_cm: Option<i32>,
    pub movement_energy: Option<i16>,
    pub stationary_distance_cm: Option<i32>,
    pub stationary_energy: Option<i16>,
    pub detection_distance_cm: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingFilters {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
