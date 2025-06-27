use sqlx::{Pool, Sqlite, SqlitePool};
use std::env;
use uuid::Uuid;
use time::OffsetDateTime;
use thiserror::Error;
use crate::models;

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

#[derive(Debug, sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    start_time: OffsetDateTime,
    end_time: OffsetDateTime,
    capacity: u32,
    location: Option<String>,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl From<EventRow> for models::Event<models::Open> {
    fn from(row: EventRow) -> Self {
        models::Event {
            id: row.id,
            name: row.name,
            description: row.description,
            start_time: row.start_time,
            end_time: row.end_time,
            capacity: row.capacity,
            location: row.location,
            created_at: row.created_at,
            updated_at: row.updated_at,
            status: models::Open,
        }
    }
}

impl From<EventRow> for models::Event<models::Full> {

    fn from(row: EventRow) -> Self {
        models::Event {
            id: row.id,
            name: row.name,
            description: row.description,
            start_time: row.start_time,
            end_time: row.end_time,
            capacity: row.capacity,
            location: row.location,
            created_at: row.created_at,
            updated_at: row.updated_at,
            status: models::Full,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ReservationRow {
    id: Uuid,
    event_id: Uuid,
    user_name: String,
    user_email: String,
    status: String,
    reservation_token: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    verified_at: Option<OffsetDateTime>,
}



// Conversion from database row to typed reservation
impl From<ReservationRow> for models::PendingReservation {
    fn from(row: ReservationRow) -> Self {
        models::Reservation {
            id: row.id,
            event_id: row.event_id,
            user_name: row.user_name,
            user_email: row.user_email,
            reservation_token: row.reservation_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
            status: models::Pending,
        }
    }
}

impl From<ReservationRow> for models::ConfirmedReservation {
    fn from(row: ReservationRow) -> Self {
        models::Reservation {
            id: row.id,
            event_id: row.event_id,
            user_name: row.user_name,
            user_email: row.user_email,
            reservation_token: row.reservation_token,
            created_at: row.created_at,
            updated_at: row.updated_at,
            status: models::Confirmed { verified_at: row.verified_at.unwrap() },
        }
    }
}


// Helper function to convert from ReservationRow to appropriate type
pub fn reservation_from_row(row: ReservationRow) -> Box<dyn std::any::Any> {
    match (row.status.as_str(), row.verified_at.is_some()) {
        ("pending", _) => Box::new(models::PendingReservation::from(row)),
        ("confirmed", true) => Box::new(models::ConfirmedReservation::from(row)),
        _ => Box::new(models::PendingReservation::from(row)), // Default to pending
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
    /// TODO have 1 version of migrations...
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

        Ok(())
    }

    /// Look up an event by ID
    pub async fn get_open_event_by_id(&self, event_id: &Uuid) -> Result<models::OpenEvent, DatabaseError> {
        let event = sqlx::query_as::<_, EventRow>(
            "SELECT * FROM events WHERE id = ? AND status = 'open'"
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::EventNotFound)?;

        Ok(event.into())
    }

    /// Get all events
    pub async fn get_all_open_events(&self) -> Result<Vec<models::OpenEvent>, DatabaseError> {
        let events = sqlx::query_as::<_, EventRow>(
            "SELECT * FROM events WHERE status = 'open' ORDER BY start_time ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    /// Insert a new reservation
    pub async fn insert_reservation(
        &self,
        event_id: &Uuid,
        user_name: &str,
        user_email: &str,
        reservation_token: &str,
    ) -> Result<models::PendingReservation, DatabaseError> {
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
    pub async fn count_event_reservations(&self, event_id: &Uuid) -> Result<u32, DatabaseError> {
        let count: u32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reservations WHERE event_id = ? AND status = 'confirmed'"
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get pending reservation by ID
    pub async fn get_pending_reservation_by_id(&self, reservation_id: &Uuid) -> Result<models::PendingReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE id = ? AND status = 'pending'"
        )
        .bind(reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(models::PendingReservation::from(row))
    }

    /// Get confirmed reservation by ID
    pub async fn get_confirmed_reservation_by_id(&self, reservation_id: &Uuid) -> Result<models::ConfirmedReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE id = ? AND status = 'confirmed'"
        )
        .bind(reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(models::ConfirmedReservation::from(row))
    }

    /// Get pending reservation by token
    pub async fn get_pending_reservation_by_token(&self, token: &str) -> Result<models::PendingReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE reservation_token = ? AND status = 'pending'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(models::PendingReservation::from(row))
    }

    /// Get confirmed reservation by token
    pub async fn get_confirmed_reservation_by_token(&self, token: &str) -> Result<models::ConfirmedReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT * FROM reservations WHERE reservation_token = ? AND status = 'confirmed'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        Ok(models::ConfirmedReservation::from(row))
    }

    /// Check if event has available capacity
    pub async fn check_open_event_capacity(&self, event_id: &Uuid) -> Result<bool, DatabaseError> {
        let event = self.get_open_event_by_id(event_id).await?;
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
    ) -> Result<models::OpenEvent, DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let event_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO events (id, name, description, start_time, end_time, capacity, location, created_at, updated_at, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'open')
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

        self.get_open_event_by_id(&event_id).await
    }

    // Helper methods for API compatibility (string IDs)
    
    /// Look up an event by string ID (converts to UUID)
    pub async fn get_open_event_by_string_id(&self, event_id: &str) -> Result<models::OpenEvent, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.get_open_event_by_id(&uuid).await
    }

    /// Insert a reservation with string event ID
    pub async fn insert_reservation_with_string_event_id(
        &self,
        event_id: &str,
        user_name: &str,
        user_email: &str,
        reservation_token: &str,
    ) -> Result<models::PendingReservation, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.insert_reservation(&uuid, user_name, user_email, reservation_token).await
    }

    /// Check event capacity with string ID
    // pub async fn check_event_capacity_with_string_id(&self, event_id: &str) -> Result<bool, DatabaseError> {
    //     let uuid = Uuid::parse_str(event_id)
    //         .map_err(|_| DatabaseError::EventNotFound)?;
    //     self.check_event_capacity(&uuid).await
    // }

    /// Confirm a pending reservation (type-safe state transition)
    pub async fn confirm_reservation(&self, pending: models::PendingReservation) -> Result<models::ConfirmedReservation, DatabaseError> {
        let confirmed = pending.confirm();
        
        sqlx::query(
            "UPDATE reservations SET status = 'confirmed', updated_at = ?, verified_at = ? WHERE id = ?"
        )
        .bind(confirmed.updated_at)
        .bind(confirmed.status.verified_at)
        .bind(&confirmed.id)
        .execute(&self.pool)
        .await?;

        Ok(confirmed)
    }

    // /// Cancel a pending reservation (type-safe state transition)
    // pub async fn cancel_pending_reservation(&self, pending: models::PendingReservation) -> Result<models::CancelledReservation, DatabaseError> {
    //     let cancelled = pending.cancel();
        
    //     sqlx::query(
    //         "UPDATE reservations SET status = 'cancelled', updated_at = ? WHERE id = ?"
    //     )
    //     .bind(cancelled.updated_at)
    //     .bind(&cancelled.id)
    //     .execute(&self.pool)
    //     .await?;

    //     Ok(cancelled)
    // }

    // /// Cancel a confirmed reservation (type-safe state transition)
    // pub async fn cancel_confirmed_reservation(&self, confirmed: models::ConfirmedReservation) -> Result<models::CancelledReservation, DatabaseError> {
    //     let cancelled = confirmed.cancel();
        
    //     sqlx::query(
    //         "UPDATE reservations SET status = 'cancelled', updated_at = ? WHERE id = ?"
    //     )
    //     .bind(cancelled.updated_at)
    //     .bind(&cancelled.id)
    //     .execute(&self.pool)
    //     .await?;

    //     Ok(cancelled)
    // }

    // Get any reservation by token with dynamic typing
    // pub async fn get_reservation_by_token_any(&self, token: &str) -> Result<models::ReservationRow, DatabaseError> {
    //     let row = sqlx::query_as::<_, ReservationRow>(
    //         "SELECT * FROM reservations WHERE reservation_token = ?"
    //     )
    //     .bind(token)
    //     .fetch_optional(&self.pool)
    //     .await?
    //     .ok_or(DatabaseError::ReservationNotFound)?;

    //     Ok(row)
    // }
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
        assert_eq!(reservation.status.to_string(), "Pending");
        
        // Test capacity check
        let has_capacity = db.check_open_event_capacity(&event.id).await.unwrap();
        assert!(has_capacity);
        
        // Test retrieval by token
        let found = db.get_pending_reservation_by_token("test-token-123").await.unwrap();
        assert_eq!(found.id, reservation.id);
    }
}
