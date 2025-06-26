use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;
use uuid::Uuid;
use time::OffsetDateTime;
use thiserror::Error;
use serde::Serialize;
use crate::models::ReservationStatus;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Environment variable error: {0}")]
    EnvError(#[from] env::VarError),
    #[error("Event not found")]
    EventNotFound,
    #[error("Reservation not found")]
    ReservationNotFound,
}

// Database Models - Used for database operations and internal data representation
// These are separate from API models to allow independent evolution and type safety

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::iso8601")]
    pub start_time: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub end_time: OffsetDateTime,
    pub capacity: i32,
    pub location: Option<String>,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}

// Type-state pattern for reservation lifecycle
#[derive(Debug, Clone, Serialize)]
pub struct Pending;

#[derive(Debug, Clone, Serialize)]
pub struct Confirmed;

#[derive(Debug, Clone, Serialize)]
pub struct Cancelled;

// Generic reservation with type-state
#[derive(Debug, Serialize)]
pub struct Reservation<State = Pending> {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub reservation_token: String,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601::option")]
    pub verified_at: Option<OffsetDateTime>,
    #[serde(skip)]
    pub state: std::marker::PhantomData<State>,
}

// Type aliases for specific states
pub type PendingReservation = Reservation<Pending>;
pub type ConfirmedReservation = Reservation<Confirmed>;
pub type CancelledReservation = Reservation<Cancelled>;

// Raw database row for deserialization
#[derive(Debug, sqlx::FromRow)]
pub struct ReservationRow {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub status: String,
    pub reservation_token: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub verified_at: Option<OffsetDateTime>,
}

// Type-state implementations for Reservation
impl<State> Reservation<State> {
    /// Get the status for serialization
    pub fn status(&self) -> ReservationStatus {
        match std::any::type_name::<State>() {
            name if name.contains("Pending") => ReservationStatus::Pending,
            name if name.contains("Confirmed") => ReservationStatus::Confirmed,
            name if name.contains("Cancelled") => ReservationStatus::Cancelled,
            _ => ReservationStatus::Pending,
        }
    }
}

// State transition methods
impl PendingReservation {
    /// Confirm a pending reservation
    pub fn confirm(self) -> ConfirmedReservation {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_name: self.user_name,
            user_email: self.user_email,
            reservation_token: self.reservation_token,
            created_at: self.created_at,
            updated_at: OffsetDateTime::now_utc(),
            verified_at: Some(OffsetDateTime::now_utc()),
            state: std::marker::PhantomData,
        }
    }
    
    /// Cancel a pending reservation
    pub fn cancel(self) -> CancelledReservation {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_name: self.user_name,
            user_email: self.user_email,
            reservation_token: self.reservation_token,
            created_at: self.created_at,
            updated_at: OffsetDateTime::now_utc(),
            verified_at: self.verified_at,
            state: std::marker::PhantomData,
        }
    }
}

impl ConfirmedReservation {
    /// Cancel a confirmed reservation
    pub fn cancel(self) -> CancelledReservation {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_name: self.user_name,
            user_email: self.user_email,
            reservation_token: self.reservation_token,
            created_at: self.created_at,
            updated_at: OffsetDateTime::now_utc(),
            verified_at: self.verified_at,
            state: std::marker::PhantomData,
        }
    }
}

// Note: CancelledReservation has no transitions (terminal state)

// Conversion from database row to typed reservation
impl From<ReservationRow> for PendingReservation {
    fn from(row: ReservationRow) -> Self {
        Reservation {
            id: row.id,
            event_id: row.event_id,
            user_name: row.user_name,
            user_email: row.user_email,
            reservation_token: row.reservation_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
            verified_at: row.verified_at,
            state: std::marker::PhantomData,
        }
    }
}

impl From<ReservationRow> for ConfirmedReservation {
    fn from(row: ReservationRow) -> Self {
        Reservation {
            id: row.id,
            event_id: row.event_id,
            user_name: row.user_name,
            user_email: row.user_email,
            reservation_token: row.reservation_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
            verified_at: row.verified_at,
            state: std::marker::PhantomData,
        }
    }
}

impl From<ReservationRow> for CancelledReservation {
    fn from(row: ReservationRow) -> Self {
        Reservation {
            id: row.id,
            event_id: row.event_id,
            user_name: row.user_name,
            user_email: row.user_email,
            reservation_token: row.reservation_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
            verified_at: row.verified_at,
            state: std::marker::PhantomData,
        }
    }
}

// Helper function to convert from ReservationRow to appropriate type
pub fn reservation_from_row(row: ReservationRow) -> Box<dyn std::any::Any> {
    match row.status.as_str() {
        "pending" => Box::new(PendingReservation::from(row)),
        "confirmed" => Box::new(ConfirmedReservation::from(row)),
        "cancelled" => Box::new(CancelledReservation::from(row)),
        _ => Box::new(PendingReservation::from(row)), // Default to pending
    }
}

// Helper function to convert ReservationStatus to string for database storage
impl ReservationStatus {
    pub fn to_db_string(&self) -> &'static str {
        match self {
            ReservationStatus::Pending => "pending",
            ReservationStatus::Confirmed => "confirmed",
            ReservationStatus::Cancelled => "cancelled",
        }
    }
}

pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    /// Create a new database connection pool from environment variable
    pub async fn new() -> Result<Self, DatabaseError> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:quick-res.db".to_string());
        
        let pool = SqlitePool::connect(&database_url).await?;
        
        // Run migrations to ensure tables exist
        Self::run_migrations(&pool).await?;
        
        Ok(Database { pool })
    }

    /// Initialize database tables
    async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        // Create events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                start_time INTEGER NOT NULL,
                end_time INTEGER NOT NULL,
                capacity INTEGER NOT NULL,
                location TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#
        )
        .execute(pool)
        .await?;

        // Create reservations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reservations (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                user_name TEXT NOT NULL,
                user_email TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                reservation_token TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                verified_at INTEGER,
                FOREIGN KEY (event_id) REFERENCES events (id)
            )
            "#
        )
        .execute(pool)
        .await?;

        // Add verified_at column if it doesn't exist (for existing databases)
        sqlx::query(
            "ALTER TABLE reservations ADD COLUMN verified_at INTEGER"
        )
        .execute(pool)
        .await
        .ok(); // Ignore error if column already exists

        Ok(())
    }

    /// Look up an event by ID
    pub async fn get_event_by_id(&self, event_id: &Uuid) -> Result<Event, DatabaseError> {
        let event = sqlx::query_as::<_, Event>(
            "SELECT * FROM events WHERE id = ?"
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::EventNotFound)?;

        Ok(event)
    }

    /// Get all events
    pub async fn get_all_events(&self) -> Result<Vec<Event>, DatabaseError> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT * FROM events ORDER BY start_time ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    /// Insert a new reservation
    pub async fn insert_reservation(
        &self,
        event_id: &Uuid,
        user_name: &str,
        user_email: &str,
        reservation_token: &str,
    ) -> Result<Reservation<Pending>, DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let reservation_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO reservations (id, event_id, user_name, user_email, status, reservation_token, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'pending', ?, ?, ?)
            "#
        )
        .bind(&reservation_id)
        .bind(event_id)
        .bind(user_name)
        .bind(user_email)
        .bind(reservation_token)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        // Fetch the inserted reservation
        self.get_pending_reservation_by_id(&reservation_id).await
    }

    /// Count current reservations for an event (confirmed only)
    pub async fn count_event_reservations(&self, event_id: &Uuid) -> Result<i32, DatabaseError> {
        let count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reservations WHERE event_id = ? AND status = 'confirmed'"
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get reservation by ID
    pub async fn get_reservation_by_id(&self, reservation_id: &Uuid) -> Result<Reservation<Pending>, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE id = ?"
        )
        .bind(reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(PendingReservation::from(row))
    }

    /// Get pending reservation by ID
    pub async fn get_pending_reservation_by_id(&self, reservation_id: &Uuid) -> Result<PendingReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE id = ? AND status = 'pending'"
        )
        .bind(reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(PendingReservation::from(row))
    }

    /// Get confirmed reservation by ID
    pub async fn get_confirmed_reservation_by_id(&self, reservation_id: &Uuid) -> Result<ConfirmedReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE id = ? AND status = 'confirmed'"
        )
        .bind(reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(ConfirmedReservation::from(row))
    }

    /// Get pending reservation by token
    pub async fn get_pending_reservation_by_token(&self, token: &str) -> Result<PendingReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE reservation_token = ? AND status = 'pending'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(PendingReservation::from(row))
    }

    /// Get confirmed reservation by token
    pub async fn get_confirmed_reservation_by_token(&self, token: &str) -> Result<ConfirmedReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE reservation_token = ? AND status = 'confirmed'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(ConfirmedReservation::from(row))
    }

    /// Check if event has available capacity
    pub async fn check_event_capacity(&self, event_id: &Uuid) -> Result<bool, DatabaseError> {
        let event = self.get_event_by_id(event_id).await?;
        let current_reservations = self.count_event_reservations(event_id).await?;
        
        Ok(current_reservations < event.capacity)
    }

    /// Create a new event (helper for testing/seeding)
    pub async fn create_event(
        &self,
        name: &str,
        description: Option<&str>,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
        capacity: i32,
        location: Option<&str>,
    ) -> Result<Event, DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let event_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO events (id, name, description, start_time, end_time, capacity, location, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&event_id)
        .bind(name)
        .bind(description)
        .bind(start_time)
        .bind(end_time)
        .bind(capacity)
        .bind(location)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        self.get_event_by_id(&event_id).await
    }

    // Helper methods for API compatibility (string IDs)
    
    /// Look up an event by string ID (converts to UUID)
    pub async fn get_event_by_string_id(&self, event_id: &str) -> Result<Event, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.get_event_by_id(&uuid).await
    }

    /// Insert a reservation with string event ID
    pub async fn insert_reservation_with_string_event_id(
        &self,
        event_id: &str,
        user_name: &str,
        user_email: &str,
        reservation_token: &str,
    ) -> Result<Reservation<Pending>, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.insert_reservation(&uuid, user_name, user_email, reservation_token).await
    }

    /// Check event capacity with string ID
    pub async fn check_event_capacity_with_string_id(&self, event_id: &str) -> Result<bool, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.check_event_capacity(&uuid).await
    }

    /// Confirm a pending reservation (type-safe state transition)
    pub async fn confirm_reservation(&self, pending: PendingReservation) -> Result<ConfirmedReservation, DatabaseError> {
        let confirmed = pending.confirm();
        
        sqlx::query(
            "UPDATE reservations SET status = 'confirmed', updated_at = ?, verified_at = ? WHERE id = ?"
        )
        .bind(confirmed.updated_at)
        .bind(confirmed.verified_at)
        .bind(&confirmed.id)
        .execute(&self.pool)
        .await?;

        Ok(confirmed)
    }

    /// Cancel a pending reservation (type-safe state transition)
    pub async fn cancel_pending_reservation(&self, pending: PendingReservation) -> Result<CancelledReservation, DatabaseError> {
        let cancelled = pending.cancel();
        
        sqlx::query(
            "UPDATE reservations SET status = 'cancelled', updated_at = ? WHERE id = ?"
        )
        .bind(cancelled.updated_at)
        .bind(&cancelled.id)
        .execute(&self.pool)
        .await?;

        Ok(cancelled)
    }

    /// Cancel a confirmed reservation (type-safe state transition)
    pub async fn cancel_confirmed_reservation(&self, confirmed: ConfirmedReservation) -> Result<CancelledReservation, DatabaseError> {
        let cancelled = confirmed.cancel();
        
        sqlx::query(
            "UPDATE reservations SET status = 'cancelled', updated_at = ? WHERE id = ?"
        )
        .bind(cancelled.updated_at)
        .bind(&cancelled.id)
        .execute(&self.pool)
        .await?;

        Ok(cancelled)
    }

    /// Get any reservation by token with dynamic typing
    pub async fn get_reservation_by_token_any(&self, token: &str) -> Result<ReservationRow, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE reservation_token = ?"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(row)
    }
}

// Conversion traits between database and API models
impl From<crate::db::Event> for crate::models::Event {
    fn from(db_event: crate::db::Event) -> Self {
        Self {
            id: db_event.id,
            name: db_event.name,
            description: db_event.description,
            start_time: db_event.start_time,
            end_time: db_event.end_time,
            capacity: db_event.capacity,
            location: db_event.location,
            created_at: db_event.created_at,
            updated_at: db_event.updated_at,
        }
    }
}

impl From<crate::db::Event> for crate::models::EventResponse {
    fn from(db_event: crate::db::Event) -> Self {
        Self {
            id: db_event.id,
            name: db_event.name,
            description: db_event.description,
            location: db_event.location,
            start_time: db_event.start_time,
            end_time: db_event.end_time,
            capacity: db_event.capacity,
            created_at: db_event.created_at,
            updated_at: db_event.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[tokio::test]
    async fn test_database_operations() {
        // Set test database URL
        env::set_var("DATABASE_URL", "sqlite::memory:");
        
        let db = Database::new().await.unwrap();
        
        // Test event creation
        let start_time = OffsetDateTime::now_utc() + Duration::hours(1);
        let end_time = start_time + Duration::hours(2);
        
        let event = db.create_event(
            "Test Event",
            Some("A test event"),
            start_time,
            end_time,
            50,
            Some("Test Location"),
        ).await.unwrap();
        
        assert_eq!(event.name, "Test Event");
        assert_eq!(event.capacity, 50);
        
        // Test reservation creation
        let reservation = db.insert_reservation(
            &event.id,
            "John Doe",
            "john@example.com",
            "test-token-123",
        ).await.unwrap();
        
        assert_eq!(reservation.user_name, "John Doe");
        assert_eq!(reservation.status().to_db_string(), "pending");
        
        // Test capacity check
        let has_capacity = db.check_event_capacity(&event.id).await.unwrap();
        assert!(has_capacity);
        
        // Test retrieval by token
        let found = db.get_pending_reservation_by_token("test-token-123").await.unwrap();
        assert_eq!(found.id, reservation.id);
    }
}
