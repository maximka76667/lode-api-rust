# lode-api-rust

REST API for receiving and serving BME280 sensor readings (temperature, humidity, pressure). The sensor board pushes data via HTTP POST; clients fetch history or subscribe to live updates over SSE.

Firmware for the sensor board: [maximka76667/lode-stm32h723](https://github.com/maximka76667/lode-stm32h723)

## Endpoints

| Method | Path        | Description                                        |
| ------ | ----------- | -------------------------------------------------- |
| `POST` | `/readings` | Push a new reading from the board                  |
| `GET`  | `/readings` | Fetch historical readings (supports filters)       |
| `GET`  | `/sse`      | Live stream of new readings via Server-Sent Events |

## Setup

**Requirements:** Rust, `sqlx-cli`

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

Create the database and run migrations:

```bash
sqlx database create && sqlx migrate run
```

Run the server:

```bash
cargo run
```

Server listens on `http://0.0.0.0:3111`.

## Usage

**Push a reading:**

```bash
curl -X POST http://localhost:3111/readings \
  -H "Content-Type: application/json" \
  -d '{"temperature_c": 23.4, "humidity_pct": 58.2, "pressure_hpa": 1013.25}'
```

**Get history:**

```bash
curl http://localhost:3111/readings
curl "http://localhost:3111/readings?limit=10&from=2026-01-01T00:00:00Z"
```

**Subscribe to live updates:**

```bash
curl -N http://localhost:3111/sse
```

## Testing

```bash
cargo test
```

## Environment

| Variable       | Description                                                   |
| -------------- | ------------------------------------------------------------- |
| `DATABASE_URL` | SQLite connection string (default: `sqlite:./readings.db`)    |
| `SQLX_OFFLINE` | Use cached query metadata instead of live DB (`true`/`false`) |
| `RUST_LOG`     | Log level (e.g. `lode_api_rust=debug`)                        |
