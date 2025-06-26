use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
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

use db::{Database, DatabaseError};
use email::{send_confirmation, send_verification, EmailError};
use error::AppError;
use models::{CreateReservationRequest, CreateReservationResponse, EmailVerificationResponse};

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
        send_verification(email, token).await
    }
    
    async fn send_confirmation<State>(&self, email: &str, reservation: &db::Reservation<State>) -> Result<(), EmailError> {
        send_confirmation(email, reservation).await
    }
}

// Application state
#[derive(Clone)]
struct AppState {
    pool: sqlx::Pool<sqlx::Sqlite>,
    email_sender: EmailSender,
}

// Note: AppError is now defined in error.rs module

// Route handlers

/// GET /events/:id - fetch event and respond JSON
async fn get_event(
    Path(event_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = Database { pool: state.pool.clone() };
    let event = db.get_event_by_id(&event_id).await?;
    
    // Convert database event to JSON response
    let response = json!({
        "id": event.id,
        "name": event.name,
        "description": event.description,
        "start_time": event.start_time,
        "end_time": event.end_time,
        "capacity": event.capacity,
        "location": event.location,
        "created_at": event.created_at,
        "updated_at": event.updated_at
    });
    
    Ok(Json(response))
}

/// POST /reservations - validate payload, check capacity, insert pending reservation
async fn reserve(
    State(state): State<AppState>,
    Json(payload): Json<CreateReservationRequest>,
) -> Result<Json<CreateReservationResponse>, AppError> {
    // Validate payload using the From<ValidationErrors> implementation
    payload.validate()?;
    
    let event_id_str = payload.event_id.to_string();
    
    // Check if event exists and has capacity
    let db = Database { pool: state.pool.clone() };
    let event = db.get_event_by_id(&event_id_str).await?;
    let current_count = db.count_event_reservations(&event_id_str).await?;
    
    if current_count >= event.capacity {
        return Err(AppError::Validation("Event is at full capacity".to_string()));
    }
    
    // Generate tokens
    let verification_token = Uuid::new_v4().to_string();
    let magic_link_token = Uuid::new_v4().to_string();
    
    // For now, we'll use the verification_token as the reservation_token
    // In a real implementation, you might want separate tokens for different purposes
    let reservation_token = format!("{}-{}", verification_token, magic_link_token);
    
    // Insert pending reservation
    let reservation = db.insert_reservation(
        &event_id_str,
        &payload.user_name,
        &payload.user_email,
        &reservation_token,
    ).await?;
    
    // Send verification email
    state.email_sender.send_verification(&payload.user_email, &verification_token).await?;
    
    let response = CreateReservationResponse {
        id: Uuid::parse_str(&reservation.id)?,
        status: reservation.status().to_string(),
    };
    
    Ok(Json(response))
}

/// GET /verify/:token - update reservation status to confirmed, send confirmation email
async fn verify_email(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<EmailVerificationResponse>, AppError> {
    let db = Database { pool: state.pool.clone() };
    
    // Find pending reservation by token
    let pending_reservation = match db.get_pending_reservation_by_token(&token).await {
        Ok(res) => res,
        Err(_) => {
            // If not found, try to find by token prefix (verification part)
            let row = sqlx::query_as::<_, db::ReservationRow>(
                "SELECT * FROM reservations WHERE reservation_token LIKE ? AND status = 'pending'"
            )
            .bind(format!("{}%", token))
            .fetch_optional(&state.pool)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| {
                // Check if there's already a confirmed reservation with this token
                let _ = sqlx::query(
                    "SELECT 1 FROM reservations WHERE reservation_token LIKE ? AND status = 'confirmed'"
                )
                .bind(format!("{}%", token))
                .fetch_optional(&state.pool);
                
                AppError::Validation("Reservation not found or already confirmed".to_string())
            })?;
            
            db::PendingReservation::from(row)
        }
    };
    
    // Store data before moving the reservation into confirm_reservation
    let reservation_id = Uuid::parse_str(&pending_reservation.id)?;
    let event_id = pending_reservation.event_id.clone();
    let user_name = pending_reservation.user_name.clone();
    let user_email = pending_reservation.user_email.clone();
    
    // Confirm the reservation using type-safe state transition
    let confirmed_reservation = db.confirm_reservation(pending_reservation).await?;
    
    // Send confirmation email
    state.email_sender.send_confirmation(&user_email, &confirmed_reservation).await?;
    
    let response = EmailVerificationResponse {
        message: "Reservation confirmed successfully".to_string(),
        status: confirmed_reservation.status().to_string(),
        reservation_id,
        event_id,
        user_name,
        verified_at: confirmed_reservation.updated_at,
    };
    
    Ok(Json(response))
}

/// POST /events - create a randomly generated event
async fn create_event(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
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
    
    let response = json!({
        "id": event.id,
        "name": event.name,
        "description": event.description,
        "start_time": event.start_time,
        "end_time": event.end_time,
        "capacity": event.capacity,
        "location": event.location,
        "created_at": event.created_at,
        "updated_at": event.updated_at
    });
    
    Ok(Json(response))
}

/// GET /reservation/:magic_token - fetch confirmed reservation, return JSON (404 otherwise)
async fn get_reservation_by_magic_token(
    Path(magic_token): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = Database { pool: state.pool.clone() };
    
    // First, try to find a confirmed reservation by exact token match
    let reservation = match db.get_confirmed_reservation_by_token(&magic_token).await {
        Ok(confirmed) => confirmed,
        Err(_) => {
            // If not found by exact match, try to find by token suffix (magic link part)
            // But still only look for confirmed reservations
            let row = sqlx::query_as::<_, db::ReservationRow>(
                "SELECT * FROM reservations WHERE reservation_token LIKE ? AND status = 'confirmed'"
            )
            .bind(format!("%{}", magic_token))
            .fetch_optional(&state.pool)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| {
                // Check if there's a pending reservation with this token to give a helpful error
                let _ = sqlx::query(
                    "SELECT 1 FROM reservations WHERE reservation_token LIKE ? AND status = 'pending'"
                )
                .bind(format!("%{}", magic_token))
                .fetch_optional(&state.pool);
                
                AppError::Validation("Reservation must be confirmed before it can be retrieved. Please check your email for the verification link.".to_string())
            })?;
            
            db::ConfirmedReservation::from(row)
        }
    };
    
    let response = json!({
        "id": reservation.id,
        "event_id": reservation.event_id,
        "user_name": reservation.user_name,
        "user_email": reservation.user_email,
        "status": reservation.status(),
        "reservation_token": reservation.reservation_token,
        "created_at": reservation.created_at,
        "updated_at": reservation.updated_at
    });
    
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
        .route("/events/new", post(create_event))
        .route("/events/{id}", get(get_event))
        .route("/reserve", post(reserve))
        .route("/verify/{token}", get(verify_email))
        .route("/retrieve/{magic_token}", get(get_reservation_by_magic_token))
        .with_state(state)
        // Layer with Trace for request logging
        .layer(TraceLayer::new_for_http())
        // Layer with CORS
        .layer(CorsLayer::permissive())
        // Layer with JSON extractor limits (16MB limit)
        .layer(axum::extract::DefaultBodyLimit::max(16 * 1024 * 1024));

    Ok(router.into())
}
