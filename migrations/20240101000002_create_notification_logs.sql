CREATE TYPE notification_status AS ENUM ('pending', 'sent', 'failed', 'gone');

CREATE TABLE notification_logs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subscription_id UUID REFERENCES subscriptions(id) ON DELETE SET NULL,
    endpoint        TEXT NOT NULL,
    status          notification_status NOT NULL DEFAULT 'pending',
    status_code     SMALLINT,
    error_message   TEXT,
    payload         JSONB,
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_logs_subscription_id ON notification_logs (subscription_id);
CREATE INDEX idx_notification_logs_status ON notification_logs (status);
CREATE INDEX idx_notification_logs_created_at ON notification_logs (created_at DESC);
