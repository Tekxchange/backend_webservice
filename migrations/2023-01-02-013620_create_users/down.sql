-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS "update_user_updated_at" ON "user";
DROP FUNCTION IF EXISTS "update_updated_at";
DROP INDEX IF EXISTS "idx_user_username";
DROP INDEX IF EXISTS "idx_user_email";
DROP TABLE IF EXISTS "user";
