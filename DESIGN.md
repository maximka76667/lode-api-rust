# lode-api-rust Design

## Overview

A REST API that receives sensor readings (temperature, humidity, pressure from BME280 + human presence data from LD2410C radar) pushed from an embedded board, and serves them to clients. Supports live readings over SSE and querying historical data with filters.

## Stack

| Crate                  | Purpose                                              |
| ---------------------- | ---------------------------------------------------- |
| `axum` + `tokio`       | Web framework + async runtime                        |
| `serde` + `serde_json` | JSON serialization                                   |
| `sqlx`                 | Async PostgreSQL driver + migrations                 |
| `tower-http`           | CORS + logging middleware                            |
| `axum::response::sse`  | Server-Sent Events (built into axum, no extra crate) |

## Endpoints

### `POST /readings`

Board pushes a new reading every ~2 seconds. The reading is immediately broadcast to SSE clients and added to an in-memory buffer. The buffer is batch-inserted into the database every 30 seconds.

**Request body:**

```json
{
  "temperature_c": 23.41,
  "humidity_pct": 58.2,
  "pressure_hpa": 1013.25,
  "presence_status": 3,
  "movement_distance_cm": 120,
  "movement_energy": 75,
  "stationary_distance_cm": 200,
  "stationary_energy": 60,
  "detection_distance_cm": 120
}
```

Presence fields are optional.

**Response:** `201 Created`

---

### `GET /sse`

Server-Sent Events stream. Each time `POST /readings` is called, the reading is broadcast to all connected SSE clients immediately (before the database write). The payload uses `recorded_at` generated locally at request time.

**Event shape (JSON, server → client):**

```json
{
  "recorded_at": "2026-04-08T10:00:00Z",
  "temperature_c": 23.41,
  "humidity_pct": 58.2,
  "pressure_hpa": 1013.25,
  "presence_status": 3,
  "movement_distance_cm": 120,
  "movement_energy": 75,
  "stationary_distance_cm": 200,
  "stationary_energy": 60,
  "detection_distance_cm": 120
}
```

---

### `GET /readings`

Returns historical readings with optional filters.

**Query params:**

- `from` — ISO 8601 datetime, inclusive
- `to` — ISO 8601 datetime, inclusive
- `limit` — max number of rows to return

**Example:** `GET /readings?from=2026-04-01T00:00:00Z&to=2026-04-08T00:00:00Z&limit=500`

**Response:** array of `DbReading` (includes `id`).

---

### `GET /readings/latest`

Returns the most recent reading from the database.

## Models

```
SensorData       — shared sensor fields (temperature, humidity, pressure, radar fields)
RawReading       — incoming POST body (flattened SensorData, JSON only)
TimestampedReading — RawReading + recorded_at (used for SSE broadcast and buffer)
DbReading        — full database row (id + recorded_at + SensorData)
```

## Database

PostgreSQL via `sqlx`. Single table:

```sql
CREATE TABLE readings (
    id                      BIGSERIAL PRIMARY KEY,
    temperature             DOUBLE PRECISION NOT NULL,
    humidity                DOUBLE PRECISION NOT NULL,
    pressure                DOUBLE PRECISION NOT NULL,
    recorded_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    presence_status         SMALLINT,
    movement_distance_cm    INTEGER,
    movement_energy         SMALLINT,
    stationary_distance_cm  INTEGER,
    stationary_energy       SMALLINT,
    detection_distance_cm   INTEGER
);
```

Migrations managed with `sqlx-cli`.

## App State & Buffering

```
AppState {
    db:        PgPool,
    tx:        broadcast::Sender<TimestampedReading>,  // SSE fanout
    buffer_tx: mpsc::Sender<TimestampedReading>,       // buffer input
}
```

`spawn_buffer_task(db, buffer_rx, flush_interval)` runs a background task that:
1. Receives `TimestampedReading` values from `buffer_rx` into a local `Vec`
2. Every `flush_interval`, batch-inserts the accumulated vec into the database and clears it

In production `flush_interval` is 30 seconds. In tests it is 100ms so the buffer flushes quickly without polling.

When `POST /readings` is called:
1. A `TimestampedReading` is created with `recorded_at = Utc::now()`
2. It is sent to `tx` for immediate SSE broadcast
3. It is sent to `buffer_tx` for deferred database insertion
