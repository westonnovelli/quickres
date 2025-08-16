use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use uuid::Uuid;
use validator::Validate;
use time::{Duration, OffsetDateTime};

mod config;
mod db;
mod email;
mod error;
mod models;
mod api;

use db::{Database, DatabaseError};
use email::{EmailError};
use error::AppError;

// Email sender component
#[derive(Clone, Debug)]
struct EmailSender {
    // In a real implementation, this would contain SMTP configuration,
    // API keys for email services, etc.
    sender_name: String,
}

impl EmailSender {
    fn new() -> Self {
        EmailSender {
            sender_name: "Quick Reservations".to_string(),
        }
    }
    
    async fn send_verification(&self, email: &str, token: &str) -> Result<(), EmailError> {
        email::send_verification(email, token).await
    }
    
    async fn send_confirmation(&self, email: &str, reservation: &models::ConfirmedReservation) -> Result<(), EmailError> {
        email::send_confirmation(email, reservation).await
    }
}

// Application state
#[derive(Clone)]
struct AppState {
    pool: sqlx::Pool<sqlx::Sqlite>,
    email_sender: EmailSender,
}

// Route handlers
async fn get_event_by_id(
    Path(event_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<api::OpenEventResponse>, AppError> {
    let event_id = Uuid::parse_str(&event_id).map_err(|_| AppError::not_found())?;

    let db = Database { pool: state.pool.clone() };
    let event = db.get_open_event_by_id(&event_id).await?;

    let response = api::OpenEventResponse {
        id: event.id,
        name: event.name,
        description: event.description,
        start_time: event.start_time,
        end_time: event.end_time,
        capacity: event.capacity,
        location: event.location,
        created_at: event.created_at,
        updated_at: event.updated_at,
        status: api::EventStatus::Open,
    };
    
    Ok(Json(response))
}

async fn reserve(
    State(state): State<AppState>,
    Json(payload): Json<api::ReserveRequest>,
) -> Result<Json<api::ReserveResponse>, AppError> {
    // Validate payload using the From<ValidationErrors> implementation
    payload.validate()?;
    
    let db = Database { pool: state.pool.clone() };
    
    // Check if event exists and has capacity
    let event = db.get_open_event_by_id(&payload.event_id).await?;
    let current_count = db.count_event_reservations(&event.id).await?;
    
    if current_count > event.capacity {
        return Err(AppError::Validation("Event is at full capacity".to_string()));
    }

    if current_count + payload.spot_count > event.capacity {
        return Err(AppError::Validation("Cannot reserve this many slots for this event".to_string()));
    }
    
    // Insert pending reservation
    let reservation = db.insert_reservation(
        models::CreatingReservation::prepare(payload.event_id, payload.user_name, payload.user_email, payload.spot_count)
    ).await?;
    
    // Send verification email with the verification token, not the reservation token
    state.email_sender.send_verification(&reservation.user_email, &reservation.verification_token.0).await?;

    let response = api::ReserveResponse {
        reservation_id: reservation.id,
        status: reservation.status.into(),  
    };
    
    Ok(Json(response))
}

async fn verify_email(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<api::VerifyEmailResponse>, AppError> {
    let db = Database { pool: state.pool.clone() };
    
    // Find pending reservation by token
    let pending_reservation = match db.get_pending_reservation_by_verification_token(&token).await {
        Ok(res) => res,
        Err(_) => {
            match db.get_confirmed_reservation_by_verification_token(&token).await {
                Ok(_) =>{
                    return Err(AppError::Validation("Reservation already confirmed".to_string()));
                }
                Err(_) => {
                    return Err(AppError::not_found());
                }   
            }
        }
    };
    
    // Store data before moving the reservation into confirm_reservation
    let user_email = pending_reservation.user_email.clone();
    let event_id = pending_reservation.event_id;
    let reservation_id = pending_reservation.id;
    
    // Confirm the reservation using type-safe state transition
    let confirmed_reservation = db.confirm_reservation(pending_reservation).await?;
    
    // Send confirmation email
    state.email_sender.send_confirmation(&user_email, &confirmed_reservation).await?;

    let response = api::VerifyEmailResponse {
        event_id,
        reservation_id,
        verified_at: confirmed_reservation.status.verified_at,
    };
    
    Ok(Json(response))
}

async fn generate_random_event(
    State(state): State<AppState>,
) -> Result<Json<api::OpenEventResponse>, AppError> {
    let db = Database { pool: state.pool.clone() };
    
    // Generate random event data
    let event_names = [
        "Tech Conference 2024",
        "Summer Music Festival",
        "Art Exhibition Opening",
        "Cooking Workshop",
        "Fitness Bootcamp",
        "Business Networking Event",
        "Photography Meetup",
        "Book Club Discussion",
        "Wine Tasting Event",
        "Startup Pitch Night"
    ];
    
    let locations = [
        "Downtown Convention Center",
        "City Park Pavilion",
        "Community Center Hall",
        "Rooftop Venue",
        "Beachside Resort",
        "Historic Library",
        "Modern Art Gallery",
        "University Auditorium",
        "Hotel Conference Room",
        "Outdoor Amphitheater"
    ];
    
    let descriptions = [
        "Join us for an amazing experience!",
        "Don't miss this exciting opportunity.",
        "A unique event you won't want to miss.",
        "Connect with like-minded individuals.",
        "Learn, grow, and have fun!",
        "An unforgettable experience awaits.",
        "Discover something new today.",
        "Be part of something special.",
        "Create lasting memories with us.",
        "Experience excellence in every detail."
    ];
    
    // Use current time as seed for simple randomization
    let now = OffsetDateTime::now_utc();
    let seed = now.unix_timestamp() as usize;
    
    let name = event_names[seed % event_names.len()];
    let location = Some(locations[seed % locations.len()]);
    let description = Some(descriptions[seed % descriptions.len()]);
    
    // Set event to start in 24-48 hours and last 2-4 hours
    let start_time = now + Duration::hours(24 + (seed % 24) as i64);
    let end_time = start_time + Duration::hours(2 + (seed % 3) as i64);
    let capacity = 20 + (seed % 80) as i32; // Random capacity between 20-100
    
    // Create the event
    let event = db.create_event(
        name,
        description.as_deref(),
        start_time,
        end_time,
        capacity,
        location.as_deref(),
    ).await?;
    
    let response = api::OpenEventResponse {
        id: event.id,
        name: event.name,
        description: event.description,
        start_time: event.start_time,
        end_time: event.end_time,
        capacity: event.capacity,
        location: event.location,
        created_at: event.created_at,
        updated_at: event.updated_at,
        status: api::EventStatus::Open,
    };
    
    Ok(Json(response))
}

async fn get_reservation_by_magic_token(
    Path(reservation_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<api::RetrieveReservationResponse>, AppError> {
    let db = Database { pool: state.pool.clone() };

    let reservation_uuid = Uuid::parse_str(&reservation_id)?;

    // Try to find confirmed reservation by id
    let confirmed_reservation = match db.get_confirmed_reservation_by_id(&reservation_uuid).await {
        Ok(confirmed) => confirmed,
        Err(_) => {
            // Check if reservation exists but is still pending
            if db.get_pending_reservation_by_id(&reservation_uuid).await.is_ok() {
                return Err(AppError::Validation("Reservation must be confirmed before it can be retrieved. Please check your email for the verification link.".to_string()));
            }
            return Err(AppError::not_found());
        }
    };

    let response = api::RetrieveReservationResponse {
        reservation_id: confirmed_reservation.id,
        user_name: confirmed_reservation.user_name,
        user_email: confirmed_reservation.user_email,
        created_at: confirmed_reservation.status.created_at,
        updated_at: confirmed_reservation.status.updated_at,
        verified_at: Some(confirmed_reservation.status.verified_at),
        reservation_tokens: confirmed_reservation
            .status
            .reservation_tokens
            .clone()
            .into_iter()
            .map(|token| api::ReservationTokenResponse {
                token: token.token().to_string(),
                status: token.state_name().to_string(),
            })
            .collect(),
        status: confirmed_reservation.status.into(),
        event: {
            let event = db.get_open_event_by_id(&confirmed_reservation.event_id).await?;
            api::RetrieveReservationEventResponse {
                id: event.id,
                name: event.name,
                description: event.description,
                start_time: event.start_time,
                end_time: event.end_time,
                capacity: event.capacity,
                location: event.location,
            }
        },
    };

    Ok(Json(response))
}

async fn scan_reservation_token(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<api::ScanTokenResponse>, AppError> {
    let db = Database { pool: state.pool.clone() };
    db.mark_reservation_token_used(&token).await?;

    let response = api::ScanTokenResponse {
        token,
        status: "used".to_string(),
    };

    Ok(Json(response))
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Optionally, you can handle errors or print a message about loading the variables
    // Initialize database
    let db = Database::new().await.expect("Failed to initialize database");
    
    // Initialize email sender
    let email_sender = EmailSender::new();
    
    // Create application state with pool and email_sender
    let state = AppState {
        pool: db.pool,
        email_sender,
    };
    
    // Build Axum router with middleware layers
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/events/new", post(generate_random_event))
        .route("/events/{id}", get(get_event_by_id))
        .route("/reserve", post(reserve))
        .route("/verify/{token}", get(verify_email))
        .route("/retrieve/{magic_token}", get(get_reservation_by_magic_token))
        .route("/scan/{token}", get(scan_reservation_token))
        .with_state(state)
        // Layer with Trace for request logging
        .layer(TraceLayer::new_for_http())
        // Layer with CORS
        .layer(CorsLayer::permissive())
        // Layer with JSON extractor limits (16MB limit)
        .layer(axum::extract::DefaultBodyLimit::max(16 * 1024 * 1024));

    Ok(router.into())
}
