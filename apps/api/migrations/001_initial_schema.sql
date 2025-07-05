-- Migration 001: Initial Database Schema
-- Consolidates all table creation, indexes, and constraints
-- This migration is idempotent - can be run multiple times safely

-- =============================================================================
-- EVENTS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS events (
    -- Primary Key: UUID stored as TEXT (SQLite standard)
    id TEXT PRIMARY KEY,
    
    -- Event Details
    name TEXT NOT NULL,
    description TEXT,                    -- Optional field (Option<String> in Rust)
    location TEXT,                       -- Optional field (Option<String> in Rust)
    
    -- Timing (stored as INTEGER for Unix epoch timestamps)
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    
    -- Capacity Management
    capacity INTEGER NOT NULL CHECK (capacity > 0),
    
    -- Status Management
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'full', 'closed')),
    
    -- Audit Fields (stored as INTEGER for Unix epoch timestamps)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    
    -- Business Logic Constraints
    CHECK (end_time > start_time),       -- Event must end after it starts
    CHECK (start_time > 0),              -- Valid timestamp
    CHECK (end_time > 0)                 -- Valid timestamp
);

-- =============================================================================
-- RESERVATIONS TABLE  
-- =============================================================================

CREATE TABLE IF NOT EXISTS reservations (
    -- Primary Key: UUID stored as TEXT (SQLite standard)
    id TEXT PRIMARY KEY,
    
    -- Foreign Key to Events
    event_id TEXT NOT NULL,
    
    -- User Information (exact field names from Rust struct)
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    
    -- Reservation Management
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'cancelled')),
    verification_token TEXT NOT NULL,
    reservation_token TEXT NOT NULL,
    
    -- Audit Fields (stored as INTEGER for Unix epoch timestamps)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    verified_at INTEGER,                 -- Optional field (Option<OffsetDateTime> in Rust)
    
    -- Foreign Key Constraint
    FOREIGN KEY (event_id) REFERENCES events (id) ON DELETE CASCADE,
    
    -- Business Logic Constraints
    UNIQUE(event_id, user_email),        -- One reservation per user per event
    UNIQUE(verification_token),          -- Globally unique tokens
    UNIQUE(reservation_token),           -- Globally unique tokens
    CHECK (created_at > 0),              -- Valid timestamp
    CHECK (updated_at > 0),              -- Valid timestamp
    CHECK (updated_at >= created_at),    -- Updated time cannot be before created
    CHECK (verified_at IS NULL OR verified_at >= created_at), -- Verified time logic
    CHECK (LENGTH(user_email) > 0),      -- Non-empty email
    CHECK (LENGTH(user_name) > 0),       -- Non-empty name
    CHECK (LENGTH(reservation_token) > 0) -- Non-empty token
);

-- =============================================================================
-- PERFORMANCE INDEXES
-- =============================================================================

-- Events Table Indexes
CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
CREATE INDEX IF NOT EXISTS idx_events_end_time ON events(end_time);
CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
CREATE INDEX IF NOT EXISTS idx_events_location ON events(location) WHERE location IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_capacity ON events(capacity);
CREATE INDEX IF NOT EXISTS idx_events_status_start_time ON events(status, start_time); -- Composite for open events by time

-- Reservations Table Indexes
CREATE INDEX IF NOT EXISTS idx_reservations_event_id ON reservations(event_id);
CREATE INDEX IF NOT EXISTS idx_reservations_user_email ON reservations(user_email);
CREATE INDEX IF NOT EXISTS idx_reservations_status ON reservations(status);
CREATE INDEX IF NOT EXISTS idx_reservations_token ON reservations(reservation_token);
CREATE INDEX IF NOT EXISTS idx_reservations_verified_at ON reservations(verified_at) WHERE verified_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_reservations_event_status ON reservations(event_id, status); -- Composite for counting by status
CREATE INDEX IF NOT EXISTS idx_reservations_created_at ON reservations(created_at);
