-- Add last_read_at column to track when user last read a post in a feed
ALTER TABLE feeds ADD COLUMN last_read_at TIMESTAMPTZ;
