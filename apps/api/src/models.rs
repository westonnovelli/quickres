use std::fmt::Display;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::api;

#[derive(Debug, Clone)]
pub struct Open;

#[derive(Debug, Clone)]
pub struct Full;

#[derive(Debug, Clone)]
pub struct Finished;

#[derive(Debug)]
pub struct Event<State> {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub capacity: u32,
    pub location: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub status: State,
}

pub type OpenEvent = Event<Open>;
pub type FullEvent = Event<Full>;


#[derive(Debug, Clone)]
pub struct Pending;

impl From<Pending> for api::ReservationStatus {
    fn from(_status: Pending) -> Self {
        api::ReservationStatus::Pending
    }
}

impl Display for Pending {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pending")
    }
}

#[derive(Debug, Clone)]
pub struct Confirmed {
    pub verified_at: OffsetDateTime,
}

impl From<Confirmed> for api::ReservationStatus {
    fn from(_status: Confirmed) -> Self {
        api::ReservationStatus::Confirmed
    }
}

impl Display for Confirmed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Confirmed")
    }
}

// Generic reservation with type-state
#[derive(Debug)]
pub struct Reservation<State> where State: Display {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub verification_token: String, // used the verify the email
    pub reservation_token: String, // used for "scanning the reservation" and "checking into the event", this should only ever happen once. (may want some admin correction escape hatch tho)
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub status: State,
}

// Type aliases for specific states
pub type PendingReservation = Reservation<Pending>;
pub type ConfirmedReservation = Reservation<Confirmed>;

impl PendingReservation {
    /// Confirm a pending reservation
    pub fn confirm(self) -> Reservation<Confirmed> {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_name: self.user_name,
            user_email: self.user_email,
            reservation_token: self.reservation_token,
            created_at: self.created_at,
            updated_at: OffsetDateTime::now_utc(),
            status: Confirmed { verified_at: OffsetDateTime::now_utc() },
        }
    }
}
