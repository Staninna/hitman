/*
-- Up Migration: introduce enum type and alter games.status column
*/
BEGIN;

-- Drop the existing default (TEXT) so we can change the column type
ALTER TABLE games ALTER COLUMN status DROP DEFAULT;

-- Create enum type that represents the possible game states
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'game_status') THEN
        CREATE TYPE game_status AS ENUM ('lobby', 'in_progress', 'finished');
    END IF;
END$$;

-- Change the column type from TEXT to the new enum, casting existing values
ALTER TABLE games
    ALTER COLUMN status TYPE game_status USING status::game_status;

-- Set the new default using the enum
ALTER TABLE games ALTER COLUMN status SET DEFAULT 'lobby';

COMMIT; 