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
pub struct Creating;

impl From<Creating> for api::ReservationStatus {
    fn from(_status: Creating) -> Self {
        api::ReservationStatus::Pending
    }
}

impl Display for Creating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Creating")
    }
}

#[derive(Debug, Clone)]
pub struct Pending {
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

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
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub verified_at: OffsetDateTime,
    pub reservation_tokens: Vec<AnyReservationToken>,
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

#[derive(Debug)] // maybe should have a "new" function impl for this instead of pub String
pub struct VerificationToken(pub String);

impl VerificationToken {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Debug)]
pub struct ScannerToken(pub String);

impl ScannerToken {
    pub fn new() -> Self {
        Self(format!("s-{}", Uuid::new_v4()))
    }
}

#[derive(Debug)]
pub struct ScannerInvite {
    pub id: Uuid,
    pub event_id: Uuid,
    pub email: String,
    pub token: ScannerToken,
    pub created_at: OffsetDateTime,
}

impl ScannerInvite {
    pub fn new(event_id: Uuid, email: String, created_at: OffsetDateTime) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_id,
            email,
            token: ScannerToken::new(),
            created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Active;
#[derive(Debug, Clone)]
pub struct Used;
#[derive(Debug, Clone)]
pub struct Expired;
#[derive(Debug, Clone)]
pub struct Unknown;

// Implement Display for all token states
impl Display for Active {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Active")
    }
}

impl Display for Used {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Used")
    }
}

impl Display for Expired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expired")
    }
}

impl Display for Unknown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown")
    }
}

#[derive(Debug, Clone)]
pub struct ReservationToken<State>
where
    State: Display,
{
    pub token: String,
    pub reservation_id: Uuid,
    pub created_at: OffsetDateTime,
    pub status: State,
}

impl ReservationToken<Active> {
    pub fn new(reservation_id: Uuid, created_at: OffsetDateTime) -> Self {
        Self {
            token: format!("r-{}", Uuid::new_v4()),
            reservation_id,
            created_at,
            status: Active,
        }
    }
}

pub type ActiveReservationToken = ReservationToken<Active>;
pub type UsedReservationToken = ReservationToken<Used>;
pub type ExpiredReservationToken = ReservationToken<Expired>;
pub type UnknownReservationToken = ReservationToken<Unknown>;

// Enum to represent tokens in any state
#[derive(Debug, Clone)]
pub enum AnyReservationToken {
    Active(ReservationToken<Active>),
    Used(ReservationToken<Used>),
    Expired(ReservationToken<Expired>),
}

impl AnyReservationToken {
    /// Get the token string regardless of state
    pub fn token(&self) -> &str {
        match self {
            AnyReservationToken::Active(token) => &token.token,
            AnyReservationToken::Used(token) => &token.token,
            AnyReservationToken::Expired(token) => &token.token,
        }
    }

    /// Get the reservation ID regardless of state
    pub fn reservation_id(&self) -> Uuid {
        match self {
            AnyReservationToken::Active(token) => token.reservation_id,
            AnyReservationToken::Used(token) => token.reservation_id,
            AnyReservationToken::Expired(token) => token.reservation_id,
        }
    }

    /// Get the creation time regardless of state
    pub fn created_at(&self) -> OffsetDateTime {
        match self {
            AnyReservationToken::Active(token) => token.created_at,
            AnyReservationToken::Used(token) => token.created_at,
            AnyReservationToken::Expired(token) => token.created_at,
        }
    }
}

// Generic reservation with type-state
#[derive(Debug)]
pub struct Reservation<State>
where
    State: Display,
{
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub verification_token: VerificationToken,
    pub spot_count: u32,
    pub status: State,
}

impl ConfirmedReservation {
    /// Get all active reservation tokens with full type safety
    pub fn get_active_reservation_tokens(&self) -> Vec<ActiveReservationToken> {
        self.status
            .reservation_tokens
            .iter()
            .filter_map(|token| {
                if let AnyReservationToken::Active(active_token) = token {
                    Some(active_token)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    /// Get all used reservation tokens with full type safety
    pub fn get_used_reservation_tokens(&self) -> Vec<UsedReservationToken> {
        self.status
            .reservation_tokens
            .iter()
            .filter_map(|token| {
                if let AnyReservationToken::Used(used_token) = token {
                    Some(used_token)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    /// Get all expired reservation tokens with full type safety
    pub fn get_expired_reservation_tokens(&self) -> Vec<ExpiredReservationToken> {
        self.status
            .reservation_tokens
            .iter()
            .filter_map(|token| {
                if let AnyReservationToken::Expired(expired_token) = token {
                    Some(expired_token)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }
}

// Type aliases for specific states
pub type CreatingReservation = Reservation<Creating>;
pub type PendingReservation = Reservation<Pending>;
pub type ConfirmedReservation = Reservation<Confirmed>;

impl CreatingReservation {
    pub fn prepare(event_id: Uuid, user_name: String, user_email: String, spot_count: u32) -> Self {
        Reservation {
            id: Uuid::new_v4(),
            event_id,
            user_name,
            user_email,
            verification_token: VerificationToken::new(),
            spot_count,
            status: Creating,
        }
    }

    pub fn create(self, created_at: OffsetDateTime) -> Reservation<Pending> {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_name: self.user_name,
            user_email: self.user_email,
            verification_token: self.verification_token,
            spot_count: self.spot_count,
            status: Pending {
                created_at,
                updated_at: created_at,
            },
        }
    }
}

impl PendingReservation {
    /// Confirm a pending reservation
    pub fn confirm(self, confirmed_at: OffsetDateTime) -> Reservation<Confirmed> {
        Reservation {
            id: self.id,
            event_id: self.event_id,
            user_name: self.user_name,
            user_email: self.user_email,
            verification_token: self.verification_token,
            spot_count: self.spot_count,
            status: Confirmed {
                created_at: self.status.created_at,
                updated_at: confirmed_at,
                verified_at: confirmed_at,
                reservation_tokens: (0..self.spot_count)
                    .map(|_| {
                        AnyReservationToken::from_active(ReservationToken::new(
                            self.id,
                            confirmed_at,
                        ))
                    })
                    .collect(),
            },
        }
    }
}

// Example usage and conversion methods
impl AnyReservationToken {
    /// Convert from a typed token to an enum variant
    pub fn from_active(token: ReservationToken<Active>) -> Self {
        AnyReservationToken::Active(token)
    }

    pub fn from_used(token: ReservationToken<Used>) -> Self {
        AnyReservationToken::Used(token)
    }

    pub fn from_expired(token: ReservationToken<Expired>) -> Self {
        AnyReservationToken::Expired(token)
    }

    /// Try to extract a typed token (returns None if wrong variant)
    pub fn as_active(&self) -> Option<&ReservationToken<Active>> {
        if let AnyReservationToken::Active(token) = self {
            Some(token)
        } else {
            None
        }
    }

    pub fn as_used(&self) -> Option<&ReservationToken<Used>> {
        if let AnyReservationToken::Used(token) = self {
            Some(token)
        } else {
            None
        }
    }

    pub fn as_expired(&self) -> Option<&ReservationToken<Expired>> {
        if let AnyReservationToken::Expired(token) = self {
            Some(token)
        } else {
            None
        }
    }

    /// Check if this token is in the active state
    pub fn is_active(&self) -> bool {
        matches!(self, AnyReservationToken::Active(_))
    }

    /// Check if this token is in the used state
    pub fn is_used(&self) -> bool {
        matches!(self, AnyReservationToken::Used(_))
    }

    /// Check if this token is in the expired state
    pub fn is_expired(&self) -> bool {
        matches!(self, AnyReservationToken::Expired(_))
    }

    /// Get the state as a string representation
    pub fn state_name(&self) -> &'static str {
        match self {
            AnyReservationToken::Active(_) => "Active",
            AnyReservationToken::Used(_) => "Used",
            AnyReservationToken::Expired(_) => "Expired",
        }
    }
}

// Example of how to use this in practice:
/*
fn example_usage() {
    // Create some tokens
    let active_token = ReservationToken {
        token: "active123".to_string(),
        reservation_id: Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        status: Active,
    };

    let used_token = ReservationToken {
        token: "used456".to_string(),
        reservation_id: Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        status: Used,
    };

    // Store them in the enum
    let tokens = vec![
        AnyReservationToken::from_active(active_token),
        AnyReservationToken::from_used(used_token),
    ];

    // Create a reservation
    let reservation = Reservation {
        id: Uuid::new_v4(),
        event_id: Uuid::new_v4(),
        user_name: "John Doe".to_string(),
        user_email: "john@example.com".to_string(),
        verification_token: VerificationToken("verification123".to_string()),
        reservation_tokens: tokens,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        status: Confirmed { verified_at: OffsetDateTime::now_utc() },
    };

    // Use the typed methods
    let active_tokens = reservation.get_active_reservation_tokens();
    let used_tokens = reservation.get_used_reservation_tokens();

    // Access common properties regardless of state
    for token in reservation.get_all_tokens() {
        println!("Token: {}, Created: {}", token.token(), token.created_at());
    }
}
*/
