-- Create reservations table
CREATE TABLE reservations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    phone TEXT,
    token TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Foreign key constraint
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    
    -- Unique constraints as specified
    UNIQUE(event_id, email),
    UNIQUE(token)
);

-- Create index on event_id as specified
CREATE INDEX idx_reservations_event_id ON reservations(event_id);

-- Additional useful indices
CREATE INDEX idx_reservations_email ON reservations(email);
CREATE INDEX idx_reservations_status ON reservations(status);
CREATE INDEX idx_reservations_token ON reservations(token);
