-- create subscriptions table
CREATE TABLE subscriptions (
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name text NOT NULL,
    subscribed_at timestamptz NOT NULL
)
