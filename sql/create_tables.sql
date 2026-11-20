CREATE TYPE achievable_type AS ENUM ('add_word_to_file');

CREATE TABLE achievables (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    type achievable_type NOT NULL,
    description JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
