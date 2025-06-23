-- Wrapping the migration in a transaction in case it fails
BEGIN
;

UPDATE
    subscriptions
SET
    STATUS = 'confirmed'
WHERE
    STATUS IS NULL;

-- make status mandatory
ALTER TABLE
    subscriptions
ALTER COLUMN
    STATUS
SET
    NOT NULL;

COMMIT;
