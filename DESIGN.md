# lode-api-rust Design

## Overview

A REST API that receives BME280 sensor readings (temperature, humidity, pressure) pushed from a separate board and serves them to clients. Supports live readings over WebSocket and querying historical data with filters.

## Stack

| Crate                  | Purpose                                              |
| ---------------------- | ---------------------------------------------------- |
| `axum` + `tokio`       | Web framework + async runtime                        |
| `serde` + `serde_json` | JSON serialization                                   |
| `sqlx`                 | Async SQLite driver + migrations                     |
| `tower-http`           | Logging middleware                                   |
| `axum::response::sse`  | Server-Sent Events (built into axum, no extra crate) |

## Endpoints

### `POST /readings`

Board pushes a new reading. Inserts a row into the database.

**Request body:**

```json
{
  "temperature_c": 23.41,
  "humidity_pct": 58.2,
  "pressure_hpa": 1013.25
}
```

**Response:** `201 Created`

---

### `GET /sse`

Server-Sent Events stream. Server pushes a new reading to all connected clients every time `POST /readings` receives data. Clients never send anything — receive only. Browser auto-reconnects if the connection drops.

**Event shape (JSON, server → client):**

```json
{
  "id": 42,
  "temperature_c": 23.41,
  "humidity_pct": 58.2,
  "pressure_hpa": 1013.25,
  "recorded_at": "2026-04-08T10:00:00Z"
}
```

---

### `GET /readings`

Returns historical readings with optional filters.

**Query params:**

- `from` — ISO 8601 datetime, inclusive
- `to` — ISO 8601 datetime, inclusive
- `limit` — max number of rows to return (default: 100)

**Example:** `GET /readings?from=2026-04-01T00:00:00Z&to=2026-04-08T00:00:00Z&limit=500`

**Response:**

```json
[
  {
    "id": 42,
    "temperature_c": 23.41,
    "humidity_pct": 58.2,
    "pressure_hpa": 1013.25,
    "recorded_at": "2026-04-08T10:00:00Z"
  }
]
```

## Database

SQLite via `sqlx`. Single table:

```sql
CREATE TABLE readings (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    temperature REAL    NOT NULL,
    humidity    REAL    NOT NULL,
    pressure    REAL    NOT NULL,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Migrations managed with `sqlx-cli`.

## App State

No sensor hardware in this repo. State holds the database connection pool and a broadcast channel for pushing live readings to SSE clients:

```
AppState {
    db: SqlitePool,
    tx: tokio::sync::broadcast::Sender<SensorReading>,
}
```

When `POST /readings` inserts a row, it also sends the reading into `tx`. Each SSE handler subscribes with `tx.subscribe()` and forwards messages to its client.
