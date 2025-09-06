-- Your SQL goes here
CREATE TABLE user_session (
    id UUID PRIMARY KEY,
    jti UUID NOT NULL,
    user_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP
);

ALTER TABLE user_session ADD FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;