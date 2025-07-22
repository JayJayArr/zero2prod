-- Wrapping the migration in a transaction in case it fails
BEGIN
;

UPDATE
    subscriptions
SET
    status = 'confirmed'
WHERE
    status IS NULL;

-- make status mandatory
ALTER TABLE
    subscriptions
ALTER COLUMN
    status
SET
    NOT NULL;

COMMIT;
