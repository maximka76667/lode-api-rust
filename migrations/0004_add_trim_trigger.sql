CREATE OR REPLACE FUNCTION trim_readings() RETURNS TRIGGER AS $$
BEGIN
    DELETE FROM readings
    WHERE id IN (
        SELECT id FROM readings
        ORDER BY recorded_at DESC
        OFFSET 302400
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trim_readings_trigger
AFTER INSERT ON readings
FOR EACH ROW EXECUTE FUNCTION trim_readings();
