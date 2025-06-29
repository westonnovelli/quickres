use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;

// Define main application error
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] crate::db::DatabaseError),
    #[error("Email error: {0}")]
    Email(#[from] crate::email::EmailError),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error("Not Found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal server error")]
    InternalServerError,
}

// Implement From trait for ValidationErrors
impl From<ValidationErrors> for AppError {
    fn from(errors: ValidationErrors) -> Self {
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .map(|(field, field_errors)| {
                let messages: Vec<String> = field_errors
                    .iter()
                    .map(|error| {
                        error.message
                            .as_ref()
                            .map(|msg| format!("{}: {}", field, msg))
                            .unwrap_or_else(|| format!("{}: validation failed", field))
                    })
                    .collect();
                messages.join(", ")
            })
            .collect();
        
        AppError::Validation(error_messages.join("; "))
    }
}

// Implement IntoResponse for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            // Database errors - map specific database errors to appropriate HTTP status codes
            AppError::Database(crate::db::DatabaseError::EventNotFound) => {
                (StatusCode::NOT_FOUND, "Event not found".to_string())
            }
            AppError::Database(crate::db::DatabaseError::ReservationNotFound) => {
                (StatusCode::NOT_FOUND, "Reservation not found".to_string())
            }
            AppError::Database(_) => {
                // Log the actual error but don't expose internal details to the client
                eprintln!("Database error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            
            // Email errors - typically internal server errors
            AppError::Email(crate::email::EmailError::InvalidEmail(email)) => {
                (StatusCode::BAD_REQUEST, format!("Invalid email address: {}", email))
            }
            AppError::Email(_) => {
                eprintln!("Email error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to send email".to_string())
            }
            
            // Validation errors - client errors
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            
            // Parsing errors - client errors
            AppError::InvalidUuid(_) => {
                (StatusCode::BAD_REQUEST, "Invalid UUID format".to_string())
            }
            AppError::JsonParsing(_) => {
                (StatusCode::BAD_REQUEST, "Invalid JSON format".to_string())
            }
            
            // HTTP status errors
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = json!({
            "error": error_message,
            "status": status.as_u16()
        });

        (status, axum::response::Json(body)).into_response()
    }
}

// Helper methods for creating common errors
impl AppError {
    /// Create a validation error from a message
    pub fn validation<T: Into<String>>(msg: T) -> Self {
        AppError::Validation(msg.into())
    }
    
    /// Create a conflict error with a message
    pub fn conflict<T: Into<String>>(msg: T) -> Self {
        AppError::Conflict(msg.into())
    }
    
    /// Create a not found error
    pub fn not_found() -> Self {
        AppError::NotFound
    }
    
    /// Create an unauthorized error
    pub fn unauthorized() -> Self {
        AppError::Unauthorized
    }
    
    /// Create a forbidden error
    pub fn forbidden() -> Self {
        AppError::Forbidden
    }
    
    /// Create an internal server error
    pub fn internal_server_error() -> Self {
        AppError::InternalServerError
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use validator::{Validate, ValidationErrors};
    use serde_json::json;
    
    #[derive(Debug, Validate)]
    struct TestStruct {
        #[validate(length(min = 1, message = "Name is required"))]
        name: String,
        #[validate(email(message = "Invalid email format"))]
        email: String,
    }
    
    #[test]
    fn test_validation_error_conversion() {
        let test_struct = TestStruct {
            name: "".to_string(),
            email: "invalid-email".to_string(),
        };
        
        let validation_errors = test_struct.validate().unwrap_err();
        let app_error: AppError = validation_errors.into();
        
        match app_error {
            AppError::Validation(msg) => {
                assert!(msg.contains("Name is required"));
                assert!(msg.contains("Invalid email format"));
            }
            _ => panic!("Expected Validation error"),
        }
    }
    
    #[test]
    fn test_database_error_conversion() {
        let db_error = crate::db::DatabaseError::EventNotFound;
        let app_error: AppError = db_error.into();
        
        match app_error {
            AppError::Database(crate::db::DatabaseError::EventNotFound) => {},
            _ => panic!("Expected Database error"),
        }
    }
    
    #[test]
    fn test_uuid_error_conversion() {
        let uuid_error = uuid::Uuid::parse_str("invalid-uuid").unwrap_err();
        let app_error: AppError = uuid_error.into();
        
        match app_error {
            AppError::InvalidUuid(_) => {},
            _ => panic!("Expected InvalidUuid error"),
        }
    }
    
    #[test]
    fn test_json_error_conversion() {
        let invalid_json = "{invalid json";
        let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let app_error: AppError = json_error.into();
        
        match app_error {
            AppError::JsonParsing(_) => {},
            _ => panic!("Expected JsonParsing error"),
        }
    }
    
    #[test]
    fn test_into_response_status_codes() {
        // Test validation error
        let validation_error = AppError::validation("Test validation error");
        let response = validation_error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        // Test not found error
        let not_found_error = AppError::not_found();
        let response = not_found_error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        // Test unauthorized error
        let unauthorized_error = AppError::unauthorized();
        let response = unauthorized_error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        // Test forbidden error
        let forbidden_error = AppError::forbidden();
        let response = forbidden_error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        
        // Test conflict error
        let conflict_error = AppError::conflict("Resource conflict");
        let response = conflict_error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        
        // Test internal server error
        let internal_error = AppError::internal_server_error();
        let response = internal_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    #[test]
    fn test_helper_methods() {
        // Test validation helper
        let error = AppError::validation("Test message");
        match error {
            AppError::Validation(msg) => assert_eq!(msg, "Test message"),
            _ => panic!("Expected Validation error"),
        }
        
        // Test conflict helper
        let error = AppError::conflict("Resource exists");
        match error {
            AppError::Conflict(msg) => assert_eq!(msg, "Resource exists"),
            _ => panic!("Expected Conflict error"),
        }
    }
}
