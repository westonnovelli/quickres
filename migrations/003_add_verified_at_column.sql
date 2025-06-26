-- Add verified_at column to reservations table
-- This column will be set when the /verify endpoint is called
ALTER TABLE reservations ADD COLUMN verified_at INTEGER;

-- Create index on verified_at for efficient querying
CREATE INDEX idx_reservations_verified_at ON reservations(verified_at);
