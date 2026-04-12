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

**Requirements:** Rust

Create a `.env` file in the project root:

```env
DATABASE_URL=postgresql://user:password@host/dbname?sslmode=require
TEST_DATABASE_URL=postgresql://user:password@host/dbname?sslmode=require
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

Tests run against a separate database specified by `TEST_DATABASE_URL`. Each test truncates the table before running.

```bash
cargo test
```

## Deployment

The app is deployed on [Koyeb](https://koyeb.com) using the included `Dockerfile`. The database is hosted on [Neon](https://neon.tech) (PostgreSQL).

Set the `DATABASE_URL` environment variable in the Koyeb dashboard to the Neon production connection string.

## Design Decisions

**PostgreSQL over SQLite** — Originally used SQLite for simplicity, but switched to PostgreSQL to enable free cloud deployment. SQLite is file-based, and free hosting services either don't provide persistent storage or require a paid plan for it. PostgreSQL can be hosted separately (Neon) and connected to from any backend host.

**Neon for database hosting** — Free tier, no credit card required, supports PostgreSQL with TLS. Provides separate projects/databases, which allows a clean split between production and test databases without any local setup.

**Koyeb for backend hosting** — Free tier, no credit card required, supports Docker deployments. The included `Dockerfile` uses a multi-stage build to keep the final image small.

**Separate `TEST_DATABASE_URL`** — Tests use a dedicated Neon project instead of the production database. `#[sqlx::test]` was considered but dropped — it dynamically creates and drops databases per test, which conflicts with Neon's connection pooling (lingering connections block the DROP). Instead, tests use a single shared database with a `TRUNCATE` at the start of each test and `max_connections(1)` to stay within Neon's free tier connection limit.

**`RUST_TEST_THREADS = "1"` in `.cargo/config.toml`** — Tests share a single database, so running them in parallel causes race conditions. This enforces sequential execution without needing to pass `--test-threads=1` manually every time.

## Environment

| Variable           | Description                                      |
| ------------------ | ------------------------------------------------ |
| `DATABASE_URL`     | PostgreSQL connection string (required)          |
| `TEST_DATABASE_URL`| PostgreSQL connection string used for tests      |
| `RUST_LOG`         | Log level (e.g. `lode_api_rust=debug`)           |
