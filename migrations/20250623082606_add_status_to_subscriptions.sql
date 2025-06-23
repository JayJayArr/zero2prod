-- add Status to the subscriptions table
ALTER TABLE
    subscriptions
ADD
    COLUMN STATUS TEXT NULL;
