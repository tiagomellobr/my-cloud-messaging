CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE subscriptions (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    endpoint     TEXT NOT NULL,
    p256dh       TEXT NOT NULL,
    auth         TEXT NOT NULL,
    tags         TEXT[] NOT NULL DEFAULT '{}',
    metadata     JSONB,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    CONSTRAINT subscriptions_endpoint_unique UNIQUE (endpoint)
);

CREATE INDEX idx_subscriptions_tags ON subscriptions USING GIN (tags);
CREATE INDEX idx_subscriptions_created_at ON subscriptions (created_at);
