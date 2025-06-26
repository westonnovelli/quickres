use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ReservationStatus {
    Confirmed,
    Pending,
    Cancelled,
}

// API Models - Used for external interfaces, validation, and serialization
// These are separate from database models to allow independent evolution

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub start_time: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub end_time: OffsetDateTime,
    pub capacity: i32,
    pub location: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reservation {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub status: ReservationStatus,
    pub reservation_token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_event_times", message = "End time must be after start time"))]
pub struct CreateEventRequest {
    #[validate(length(min = 1, max = 255, message = "Event name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    #[validate(length(max = 255, message = "Location must be less than 255 characters"))]
    pub location: Option<String>,
    #[validate(range(min = 1, max = 10000, message = "Capacity must be between 1 and 10000"))]
    pub capacity: i32,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
}

// Custom validation function for event times
fn validate_event_times(event: &CreateEventRequest) -> Result<(), validator::ValidationError> {
    if event.end_time <= event.start_time {
        return Err(validator::ValidationError::new("invalid_time_range"));
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateReservationRequest {
    pub event_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "User name must be between 1 and 255 characters"))]
    pub user_name: String,
    #[validate(email(message = "Please provide a valid email address"))]
    pub user_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReservationResponse {
    pub id: Uuid,
    pub status: ReservationStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationResponse {
    pub message: String,
    pub status: ReservationStatus,
    pub reservation_id: Uuid,
    pub event_id: Uuid,
    pub user_name: String,
    #[serde(with = "time::serde::iso8601")]
    pub verified_at: OffsetDateTime,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    #[serde(with = "time::serde::iso8601")]
    pub start_time: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub end_time: OffsetDateTime,
    pub capacity: i32,
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    pub updated_at: OffsetDateTime,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RetrieveReservationResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub event: EventResponse,
    pub user_name: String,
    pub user_email: String,
    pub status: ReservationStatus,
    pub reservation_token: String,
}

