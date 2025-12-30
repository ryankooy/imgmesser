CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS user_profile (
    username text PRIMARY KEY,
    password varchar(100) NOT NULL,
    object_base_path text NOT NULL
);

CREATE TABLE IF NOT EXISTS image (
    id uuid PRIMARY KEY,
    name text,
    extension text,
    username text NOT NULL REFERENCES user_profile(username)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS image_version (
    image_id uuid REFERENCES image(id) ON DELETE CASCADE,
    version text,
    ts timestamptz NOT NULL DEFAULT NOW(),
    latest boolean NOT NULL,
    PRIMARY KEY(image_id, version)
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    username text NOT NULL REFERENCES user_profile(username)
        ON DELETE CASCADE,
    token varchar(255) UNIQUE NOT NULL,
    created_at timestamptz DEFAULT NOW(),
    last_used_at timestamptz DEFAULT NOW(),
    expires_at timestamptz NOT NULL DEFAULT (NOW() + INTERVAL '7 days'),
    is_used boolean NOT NULL DEFAULT FALSE,
    used_at timestamptz
);

CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token
    ON refresh_tokens(token);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_username
    ON refresh_tokens(username);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_expires_at
    ON refresh_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_is_used
    ON refresh_tokens(is_used);
