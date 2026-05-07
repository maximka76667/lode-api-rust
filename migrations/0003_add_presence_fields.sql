ALTER TABLE readings
    ADD COLUMN IF NOT EXISTS presence_status        SMALLINT,
    ADD COLUMN IF NOT EXISTS movement_distance_cm   INTEGER,
    ADD COLUMN IF NOT EXISTS movement_energy        SMALLINT,
    ADD COLUMN IF NOT EXISTS stationary_distance_cm INTEGER,
    ADD COLUMN IF NOT EXISTS stationary_energy      SMALLINT,
    ADD COLUMN IF NOT EXISTS detection_distance_cm  INTEGER;
