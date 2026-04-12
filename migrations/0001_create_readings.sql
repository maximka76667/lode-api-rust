CREATE TABLE IF NOT EXISTS readings (
    id          BIGSERIAL        PRIMARY KEY,
    temperature DOUBLE PRECISION NOT NULL,
    humidity    DOUBLE PRECISION NOT NULL,
    pressure    DOUBLE PRECISION NOT NULL,
    recorded_at TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);
