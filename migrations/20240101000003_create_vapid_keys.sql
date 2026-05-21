CREATE TABLE vapid_keys (
    id              SMALLINT PRIMARY KEY DEFAULT 1,
    private_key_pem TEXT NOT NULL,
    public_key_b64  TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT vapid_keys_single_row CHECK (id = 1)
);
