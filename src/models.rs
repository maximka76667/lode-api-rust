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
}

#[derive(Debug, Deserialize)]
pub struct NewReading {
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub pressure_hpa: f64,
}

#[derive(Debug, Deserialize)]
pub struct ReadingFilters {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
