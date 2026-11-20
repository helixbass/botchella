CREATE TYPE achievable_type AS ENUM ('add_word_to_file');

CREATE TABLE achievables (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    type achievable_type NOT NULL,
    description JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE achievable_action AS ENUM ('start', 'success', 'give_up', 'failed');

CREATE TABLE achievable_attempts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    achievable_id BIGINT REFERENCES achievables(id) NOT NULL ON DELETE CASCADE,
    action achievable_action NOT NULL,
);
