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
    #[error("Reservation token not found or already used")]
    TokenInvalid,
}

// Database Models - Used for database operations and internal data representation

#[derive(Debug, sqlx::FromRow)]
struct EventRow {
    id: String,  // Store UUID as TEXT in SQLite
    name: String,
    description: Option<String>,
    start_time: OffsetDateTime,
    end_time: OffsetDateTime,
    capacity: u32,
    max_spots_per_reservation: u32,
    location: Option<String>,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl From<EventRow> for models::Event<models::Open> {
    fn from(row: EventRow) -> Self {
        models::Event {
            id: Uuid::parse_str(&row.id).expect("Invalid UUID in database"),
            name: row.name,
            description: row.description,
            start_time: row.start_time,
            end_time: row.end_time,
            capacity: row.capacity,
            max_spots_per_reservation: row.max_spots_per_reservation,
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
            id: Uuid::parse_str(&row.id).expect("Invalid UUID in database"),
            name: row.name,
            description: row.description,
            start_time: row.start_time,
            end_time: row.end_time,
            capacity: row.capacity,
            max_spots_per_reservation: row.max_spots_per_reservation,
            location: row.location,
            created_at: row.created_at,
            updated_at: row.updated_at,
            status: models::Full,
        }
    }
}


#[derive(Debug, sqlx::FromRow)]
struct ReservationRow {
    id: String,  // Store UUID as TEXT in SQLite
    event_id: String,  // Store UUID as TEXT in SQLite
    user_name: String,
    user_email: String,
    spot_count: u32,
    status: String,
    verification_token: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    verified_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct ReservationTokenRow {
    id: String,
    reservation_id: String,
    token: String,
    status: String,
    created_at: OffsetDateTime,
    used_at: Option<OffsetDateTime>,
}

impl From<String> for models::VerificationToken {
    fn from(token: String) -> Self {
        models::VerificationToken(token)
    }
}

impl ReservationTokenRow {
    fn to_active_reservation_token(self) -> models::ActiveReservationToken {
        models::ActiveReservationToken {
            token: self.token,
            reservation_id: Uuid::parse_str(&self.reservation_id).expect("Invalid UUID in database"),
            created_at: self.created_at,
            status: models::Active,
        }
    }

    fn to_used_reservation_token(self) -> models::UsedReservationToken {
        models::UsedReservationToken {
            token: self.token,
            reservation_id: Uuid::parse_str(&self.reservation_id).expect("Invalid UUID in database"),
            created_at: self.created_at,
            status: models::Used,
        }
    }   

    fn to_expired_reservation_token(self) -> models::ExpiredReservationToken {
        models::ExpiredReservationToken {
            token: self.token,
            reservation_id: Uuid::parse_str(&self.reservation_id).expect("Invalid UUID in database"),
            created_at: self.created_at,
            status: models::Expired,
        }
    }

    fn to_any_reservation_token(self) -> models::AnyReservationToken {
        match self.status.as_str() {
            "active" => {
                models::AnyReservationToken::Active(self.to_active_reservation_token())
            }
            "used" => {
                models::AnyReservationToken::Used(self.to_used_reservation_token())
            }
            "expired" => {
                models::AnyReservationToken::Expired(self.to_expired_reservation_token())
            }
            _ => {
                // Handle unknown status by treating as expired
                models::AnyReservationToken::Expired(self.to_expired_reservation_token())
            }
        }
    }
}

// Conversion from database row to typed reservation
impl ReservationRow {
    async fn get_reservation_tokens(&self, db: &Database) -> Result<Vec<models::AnyReservationToken>, DatabaseError> {
        match db.get_reservation_tokens_by_reservation_id(&self.id).await {
            Ok(tokens) => Ok(tokens),
            Err(e) => Err(e),
        }
    }

    async fn to_pending_reservation(self) -> Result<models::PendingReservation, DatabaseError> {

        Ok(models::Reservation {
            id: Uuid::parse_str(&self.id).expect("Invalid UUID in database"),
            event_id: Uuid::parse_str(&self.event_id).expect("Invalid UUID in database"),
            user_name: self.user_name,
            user_email: self.user_email,
            spot_count: self.spot_count,
            verification_token: self.verification_token.into(),
            status: models::Pending {
                created_at: self.created_at,
                updated_at: self.updated_at,
            },    
        })
    }

    async fn to_confirmed_reservation(self, db: &Database) -> Result<models::ConfirmedReservation, DatabaseError> {
        let reservation_tokens = self.get_reservation_tokens(db).await?;
        
        Ok(models::Reservation {
            id: Uuid::parse_str(&self.id).expect("Invalid UUID in database"),
            event_id: Uuid::parse_str(&self.event_id).expect("Invalid UUID in database"),
            user_name: self.user_name,
            user_email: self.user_email,
            spot_count: self.spot_count,
            verification_token: self.verification_token.into(),
            status: models::Confirmed { 
                created_at: self.created_at,
                updated_at: self.updated_at,
                verified_at: self.verified_at.unwrap(), 
                reservation_tokens,
            },
        })
    }
}


// Helper function to convert from ReservationRow to appropriate type
pub async fn reservation_from_row(row: ReservationRow, db: &Database) -> Result<Box<dyn std::any::Any>, DatabaseError> {
    match (row.status.as_str(), row.verified_at.is_some()) {
        ("pending", _) => {
            let pending = row.to_pending_reservation().await?;
            Ok(Box::new(pending))
        },
        ("confirmed", true) => {
            let confirmed = row.to_confirmed_reservation(db).await?;
            Ok(Box::new(confirmed))
        },
        _ => {
            let pending = row.to_pending_reservation().await?;
            Ok(Box::new(pending)) // Default to pending
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
        
        // Ensure you have run the necessary SQL migrations before launching the application.
        // You can use `sqlx migrate run` or initialize migrations during the application start-up using `sqlx::migrate!()` macro.
        
        Ok(Database { pool })
    }

    pub async fn get_open_event_by_id(&self, event_id: &Uuid) -> Result<models::OpenEvent, DatabaseError> {
        let event = sqlx::query_as::<_, EventRow>(
"SELECT id, name, description, start_time, end_time, capacity, max_spots_per_reservation, location, status, created_at, updated_at FROM events WHERE id = ? AND status = 'open'"
        )
        .bind(event_id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::EventNotFound)?;

        Ok(event.into())
    }

    pub async fn get_all_open_events(&self) -> Result<Vec<models::OpenEvent>, DatabaseError> {
        let events = sqlx::query_as::<_, EventRow>(
"SELECT id, name, description, start_time, end_time, capacity, max_spots_per_reservation, location, status, created_at, updated_at FROM events WHERE status = 'open' ORDER BY start_time ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(events.into_iter().map(|e| e.into()).collect())
    }

    pub async fn insert_reservation(
        &self,
        creating_reservation: models::CreatingReservation,
    ) -> Result<models::PendingReservation, DatabaseError> {
        // Insert the reservation (timestamps handled by database)
        sqlx::query(
            r#"
            INSERT INTO reservations (id, event_id, user_name, user_email, spot_count,   status, verification_token, verified_at)
            VALUES                   ( ?,        ?,         ?,          ?,          ?, 'pending',          ?,        NULL)
            "#
        )
        .bind(creating_reservation.id.to_string())
        .bind(creating_reservation.event_id.to_string())
        .bind(&creating_reservation.user_name)
        .bind(&creating_reservation.user_email)
        .bind(creating_reservation.spot_count)
        .bind(creating_reservation.verification_token.0)
        .execute(&self.pool)
        .await?;

        // TODO handle duplicate email error and surface to UI

        // Fetch the inserted reservation
        self.get_pending_reservation_by_id(&creating_reservation.id).await
    }

    pub async fn count_event_reservations(&self, event_id: &Uuid) -> Result<u32, DatabaseError> {
        let count: u32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reservations WHERE event_id = ? AND status = 'confirmed'"
        )
        .bind(event_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn get_pending_reservation_by_id(&self, reservation_id: &Uuid) -> Result<models::PendingReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
        "SELECT id, event_id, user_name, user_email, spot_count, status, verification_token, created_at, updated_at, verified_at FROM reservations WHERE id = ? AND status = 'pending'"
        )
        .bind(reservation_id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        row.to_pending_reservation().await
    }

    pub async fn get_confirmed_reservation_by_id(&self, reservation_id: &Uuid) -> Result<models::ConfirmedReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
        "SELECT id, event_id, user_name, user_email, spot_count, status, verification_token, created_at, updated_at, verified_at FROM reservations WHERE id = ? AND status = 'confirmed'"
        )
        .bind(reservation_id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        row.to_confirmed_reservation(&self).await
    }

    pub async fn get_pending_reservation_by_verification_token(&self, token: &str) -> Result<models::PendingReservation, DatabaseError> {
        println!("Getting pending reservation by verification token: {}", token);
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT id, event_id, user_name, user_email, spot_count, status, verification_token, created_at, updated_at, verified_at FROM reservations WHERE verification_token = ? AND status = 'pending'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        row.to_pending_reservation().await
    }

    pub async fn get_confirmed_reservation_by_verification_token(&self, token: &str) -> Result<models::ConfirmedReservation, DatabaseError> {
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT id, event_id, user_name, user_email, spot_count, status, verification_token, created_at, updated_at, verified_at FROM reservations WHERE verification_token = ? AND status = 'confirmed'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        row.to_confirmed_reservation(&self).await
    }

    pub async fn get_pending_reservation_by_reservation_token(&self, token: &str) -> Result<models::PendingReservation, DatabaseError> {
        println!("Getting pending reservation by reservation token: {}", token);
        
        // First find the reservation_id for this token
        let reservation_id: String = sqlx::query_scalar(
            "SELECT reservation_id FROM reservation_tokens WHERE token = ? AND status = 'active'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        // Then get the reservation
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT id, event_id, user_name, user_email, status, verification_token, created_at, updated_at, verified_at FROM reservations WHERE id = ? AND status = 'pending'"
        )
        .bind(&reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        row.to_pending_reservation().await
    }

    pub async fn get_confirmed_reservation_by_reservation_token(&self, token: &str) -> Result<models::ConfirmedReservation, DatabaseError> {
        // First find the reservation_id for this token
        let reservation_id: String = sqlx::query_scalar(
            "SELECT reservation_id FROM reservation_tokens WHERE token = ? AND status = 'active'"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        // Then get the reservation
        let row = sqlx::query_as::<_, ReservationRow>(
            "SELECT id, event_id, user_name, user_email, status, verification_token, created_at, updated_at, verified_at FROM reservations WHERE id = ? AND status = 'confirmed'"
        )
        .bind(&reservation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(DatabaseError::ReservationNotFound)?;

        row.to_confirmed_reservation(&self).await
    }

    async fn get_reservation_tokens_by_reservation_id(&self, reservation_id: &str) -> Result<Vec<models::AnyReservationToken>, DatabaseError> {
        let token_rows = sqlx::query_as::<_, ReservationTokenRow>(
            "SELECT id, reservation_id, token, status, created_at, used_at FROM reservation_tokens WHERE reservation_id = ?"
        )
        .bind(reservation_id)
        .fetch_all(&self.pool)
        .await?;
    
        Ok(token_rows.into_iter().map(|row| {
            let reservation_id = Uuid::parse_str(&row.reservation_id).expect("Invalid reservation UUID in database");
            
            // Map database status to appropriate AnyReservationToken variant
            match row.status.as_str() {
                "active" => {
                    let active_token = models::ReservationToken {
                        token: row.token,
                        reservation_id,
                        created_at: row.created_at,
                        status: models::Active,
                    };
                    models::AnyReservationToken::Active(active_token)
                }
                "used" => {
                    let used_token = models::ReservationToken {
                        token: row.token,
                        reservation_id,
                        created_at: row.created_at,
                        status: models::Used,
                    };
                    models::AnyReservationToken::Used(used_token)
                }
                "expired" => {
                    let expired_token = models::ReservationToken {
                        token: row.token,
                        reservation_id,
                        created_at: row.created_at,
                        status: models::Expired,
                    };
                    models::AnyReservationToken::Expired(expired_token)
                }
                _ => {
                    // Handle unknown status by treating as expired
                    let expired_token = models::ReservationToken {
                        token: row.token,
                        reservation_id,
                        created_at: row.created_at,
                        status: models::Expired,
                    };
                    models::AnyReservationToken::Expired(expired_token)
                }
            }
        }).collect())
    }

    pub async fn check_open_event_capacity(&self, event_id: &Uuid) -> Result<bool, DatabaseError> {
        let event = self.get_open_event_by_id(event_id).await?;
        let current_reservations = self.count_event_reservations(event_id).await?;
        
        Ok(current_reservations < event.capacity)
    }

    pub async fn create_event(
        &self,
        name: &str,
        description: Option<&str>,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
        capacity: i32,
        max_spots_per_reservation: i32,
        location: Option<&str>,
    ) -> Result<models::OpenEvent, DatabaseError> {
        let event_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO events (id, name, description, start_time, end_time, capacity, max_spots_per_reservation, location, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'open')
            "#
        )
        .bind(event_id.to_string())
        .bind(name)
        .bind(description)
        .bind(start_time)
        .bind(end_time)
        .bind(capacity)
        .bind(max_spots_per_reservation)
        .bind(location)
        .execute(&self.pool)
        .await?;

        self.get_open_event_by_id(&event_id).await
    }

    pub async fn update_event(
        &self,
        event_id: &Uuid,
        name: &str,
        description: Option<&str>,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
        capacity: i32,
        max_spots_per_reservation: i32,
        location: Option<&str>,
    ) -> Result<models::OpenEvent, DatabaseError> {
        sqlx::query(
            r#"
            UPDATE events SET name = ?, description = ?, start_time = ?, end_time = ?, capacity = ?, max_spots_per_reservation = ?, location = ?
            WHERE id = ? AND status = 'open'
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(start_time)
        .bind(end_time)
        .bind(capacity)
        .bind(max_spots_per_reservation)
        .bind(location)
        .bind(event_id.to_string())
        .execute(&self.pool)
        .await?;

        self.get_open_event_by_id(event_id).await
    }

    // Helper methods for API compatibility (string IDs)
    
    /// Look up an event by string ID (converts to UUID)
    pub async fn get_open_event_by_string_id(&self, event_id: &str) -> Result<models::OpenEvent, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.get_open_event_by_id(&uuid).await
    }

    pub async fn check_event_capacity_with_string_id(&self, event_id: &str) -> Result<bool, DatabaseError> {
        let uuid = Uuid::parse_str(event_id)
            .map_err(|_| DatabaseError::EventNotFound)?;
        self.check_open_event_capacity(&uuid).await
    }

    /// Confirm a pending reservation (type-safe state transition)
    pub async fn confirm_reservation(&self, pending: models::PendingReservation) -> Result<models::ConfirmedReservation, DatabaseError> {
        let confirmed = pending.confirm(OffsetDateTime::now_utc());
        
        // Update the reservation status and set verified_at timestamp
        // Note: updated_at is handled by database trigger, verified_at is set by application
        sqlx::query(
            "UPDATE reservations SET status = 'confirmed', verified_at = ? WHERE id = ?"
        )
        .bind(confirmed.status.verified_at)
        .bind(confirmed.id.to_string())
        .execute(&self.pool)
        .await?;

        // Insert the reservation tokens
        for token in &confirmed.status.reservation_tokens {
            let token_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO reservation_tokens (id, reservation_id, token, status)
                VALUES (?, ?, ?, 'active')
                "#
            )
            .bind(token_id.to_string())
            .bind(confirmed.id.to_string())
            .bind(&token.token())
            .execute(&self.pool)
            .await?;
        }

        Ok(confirmed)
    }

    pub async fn mark_reservation_token_used(&self, token: &str) -> Result<(), DatabaseError> {
        let now = OffsetDateTime::now_utc();
        let result = sqlx::query(
            "UPDATE reservation_tokens SET status = 'used', used_at = ? WHERE token = ? AND status = 'active'",
        )
        .bind(now)
        .bind(token)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatabaseError::TokenInvalid);
        }

        Ok(())
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
    //         "SELECT id, event_id, user_name, user_email, status, reservation_token, created_at, updated_at, verified_at FROM reservations WHERE reservation_token = ?"
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
        
        // Run migrations for test database
        sqlx::migrate!("./migrations")
            .run(&db.pool)
            .await
            .expect("Failed to run migrations");
        
        // Test event creation
        let start_time = OffsetDateTime::now_utc() + Duration::hours(1);
        let end_time = start_time + Duration::hours(2);
        
        let event = db.create_event(
            "Test Event",
            Some("A test event"),
            start_time,
            end_time,
            50,
            5,
            Some("Test Location"),
        ).await.unwrap();
        
        assert_eq!(event.name, "Test Event");
        assert_eq!(event.capacity, 50);
        
        // Test reservation creation
        let creating_reservation = models::CreatingReservation {
            id: Uuid::new_v4(),
            event_id: event.id,
            user_name: "John Doe".to_string(),
            user_email: "john@example.com".to_string(),
            spot_count: 1,
            verification_token: models::VerificationToken::new(),
            status: models::Creating,
        };
        
        let reservation = db.insert_reservation(creating_reservation).await.unwrap();

        assert_eq!(reservation.user_name, "John Doe");
        assert_eq!(reservation.status.to_string(), "Pending");
        
        // Test capacity check
        let has_capacity = db.check_open_event_capacity(&event.id).await.unwrap();
        assert!(has_capacity);
        
        // Test retrieval by verification token
        let found = db.get_pending_reservation_by_verification_token("verification-token-123").await.unwrap();
        assert_eq!(found.id, reservation.id);
    }
}
