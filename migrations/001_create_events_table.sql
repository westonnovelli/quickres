-- Create events table
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    location TEXT,
    start_time TEXT NOT NULL, -- ISO 8601 datetime string
    end_time TEXT NOT NULL,   -- ISO 8601 datetime string
    capacity INTEGER NOT NULL,
    available_spots INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'open'
);

-- Create index on commonly queried fields
CREATE INDEX idx_events_start_time ON events(start_time);
CREATE INDEX idx_events_location ON events(location);
CREATE INDEX idx_events_capacity ON events(capacity);
