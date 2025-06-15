-- create subscriptions table
CREATE TABLE subscriptions (
  id uuid not null,
  primary key (id),
  email TEXT not null unique,
  name text not null,
  subscribed_at timestamptz not null
)
