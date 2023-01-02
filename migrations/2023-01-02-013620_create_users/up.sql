-- Your SQL goes here
CREATE TABLE "user" (
    "id" BIGSERIAL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "username" VARCHAR(32) NOT NULL,
    "email" VARCHAR(512) NOT NULL,
    "password" VARCHAR(512) NOT NULL
);

CREATE UNIQUE INDEX idx_user_username ON "user" (username);
CREATE UNIQUE INDEX idx_user_email ON "user" (email);

CREATE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_user_updated_at
    BEFORE UPDATE
    ON
        "user"
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at();