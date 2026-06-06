use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SensorData {
    #[serde(rename = "temperature_c")]
    pub temperature: f64,
    #[serde(rename = "humidity_pct")]
    pub humidity: f64,
    #[serde(rename = "pressure_hpa")]
    pub pressure: f64,
    pub presence_status: Option<i16>,
    pub movement_distance_cm: Option<i32>,
    pub movement_energy: Option<i16>,
    pub stationary_distance_cm: Option<i32>,
    pub stationary_energy: Option<i16>,
    pub detection_distance_cm: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RawReading {
    #[serde(flatten)]
    pub data: SensorData,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimestampedReading {
    pub recorded_at: DateTime<Utc>,
    #[serde(flatten)]
    pub data: SensorData,
}

impl From<RawReading> for TimestampedReading {
    fn from(r: RawReading) -> Self {
        Self {
            recorded_at: Utc::now(),
            data: r.data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbReading {
    pub id: i64,
    pub recorded_at: DateTime<Utc>,
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub data: SensorData,
}

#[derive(Debug, Deserialize)]
pub struct ReadingFilters {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
