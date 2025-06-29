use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_event_times", message = "End time must be after start time"))]
pub struct OpenEventRequest {
    #[validate(length(min = 1, max = 255, message = "Event name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    #[validate(length(max = 255, message = "Location must be less than 255 characters"))]
    pub location: Option<String>,
    #[validate(range(min = 1, max = 10000, message = "Capacity must be between 1 and 10000"))]
    pub capacity: u32,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
}

// Custom validation function for event times
fn validate_event_times(event: &OpenEventRequest) -> Result<(), validator::ValidationError> {
    if event.end_time <= event.start_time {
        return Err(validator::ValidationError::new("invalid_time_range"));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub enum EventStatus {
    Open,
    Full,
    Finished,
}

#[derive(Debug, Serialize)]
pub struct OpenEventResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub capacity: u32,
    pub location: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub status: EventStatus,
}


#[derive(Debug, Deserialize, Validate)]
pub struct ReserveRequest {
    pub event_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub user_name: String,
    #[validate(email(message = "Invalid email address"))]
    pub user_email: String,
}

#[derive(Debug, Serialize)]
pub struct ReserveResponse {
    pub reservation_id: Uuid,
    pub status: ReservationStatus,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    pub event_id: Uuid,
    pub reservation_id: Uuid,
    pub verified_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub enum ReservationStatus {
    Pending,
    Confirmed,
}

#[derive(Debug, Serialize)]

pub struct RetrieveReservationResponse {
    pub reservation_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub status: ReservationStatus,
    pub reservation_token: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub verified_at: Option<OffsetDateTime>,
    pub event: RetrieveReservationEventResponse,
}

#[derive(Debug, Serialize)]
pub struct RetrieveReservationEventResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub capacity: u32,
    pub location: Option<String>,
}