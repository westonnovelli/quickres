
use thiserror::Error;
use std::env;
use crate::models;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Failed to send email: {0}")]
    SendFailure(String),
    #[error("Invalid email address: {0}")]
    InvalidEmail(String),
}

/// Send a verification email with a token
/// Currently logs to stdout, but designed to be pluggable for real email providers
pub async fn send_verification(email: &str, token: &str) -> Result<(), EmailError> {
    // Validate email format (basic validation)
    if !is_valid_email(email) {
        return Err(EmailError::InvalidEmail(email.to_string()));
    }

    // Get configuration from environment variables
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let email_from = env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@quick-res.example.com".to_string());
    let email_from_name = env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Quick Reservations".to_string());
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "Quick Reservations".to_string());
    
    // Build verification URL
    let verification_url = format!("{}/verify/{}", base_url, token);

    // For now, log to stdout - this will be replaced with actual email provider integration
    println!("=== EMAIL VERIFICATION ===");
    println!("From: {} <{}>", email_from_name, email_from);
    println!("To: {}", email);
    println!("Subject: Verify your email address for {}", app_name);
    println!("Body:");
    println!("Please verify your email address by clicking the following link:");
    println!("{}", verification_url);
    println!("If you did not request this verification, please ignore this email.");
    println!("This link will expire in 24 hours for security reasons.");
    println!("========================");

    // Simulate potential email sending failure for testing
    // In a real implementation, this would handle actual SMTP errors, API failures, etc.
    Ok(())
}

/// Send a confirmation email for a reservation
/// Currently logs to stdout, but designed to be pluggable for real email providers
pub async fn send_confirmation(email: &str, reservation: &models::ConfirmedReservation) -> Result<(), EmailError> {
    // Validate email format (basic validation)
    if !is_valid_email(email) {
        return Err(EmailError::InvalidEmail(email.to_string()));
    }

    // Get configuration from environment variables
    let base_url = env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let email_from = env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@quick-res.example.com".to_string());
    let email_from_name = env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Quick Reservations".to_string());
    let app_name = env::var("APP_NAME").unwrap_or_else(|_| "Quick Reservations".to_string());
    
    // Build magic link URL
    let magic_link_url = format!("{}/retrieve/{}", base_url, reservation.reservation_token);

    // For now, log to stdout - this will be replaced with actual email provider integration
    println!("=== RESERVATION CONFIRMATION ===");
    println!("From: {} <{}>", email_from_name, email_from);
    println!("To: {}", email);
    println!("Subject: Reservation Confirmed - {}", app_name);
    println!("Body:");
    println!("Dear {},", reservation.user_name);
    println!("");
    println!("Your reservation has been confirmed!");
    println!("Reservation Details:");
    println!("- Reservation ID: {}", reservation.id);
    println!("- Event ID: {}", reservation.event_id);
    println!("- Status: {}", reservation.status);
    println!("- Created: {}", reservation.created_at);
    println!("");
    println!("Access your reservation details at:");
    println!("{}", magic_link_url);
    println!("");
    println!("Thank you for using {}!", app_name);
    println!("==============================");

    // Simulate potential email sending failure for testing
    // In a real implementation, this would handle actual SMTP errors, API failures, etc.
    Ok(())
}

/// Basic email validation
/// In a production system, you might want to use a more robust email validation library
fn is_valid_email(email: &str) -> bool {
    if email.len() <= 5 {
        return false;
    }
    
    let at_count = email.matches('@').count();
    if at_count != 1 {
        return false;
    }
    
    let at_pos = email.find('@').unwrap();
    let local_part = &email[..at_pos];
    let domain_part = &email[at_pos + 1..];
    
    // Basic checks
    !local_part.is_empty() 
        && !domain_part.is_empty() 
        && domain_part.contains('.') 
        && !domain_part.starts_with('.') 
        && !domain_part.ends_with('.')
}

#[cfg(test)]
mod tests {
    use crate::models;

    use super::*;
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_send_verification_valid_email() {
        let result = send_verification("test@example.com", "abc123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_verification_invalid_email() {
        let result = send_verification("invalid-email", "abc123").await;
        assert!(result.is_err());
        match result {
            Err(EmailError::InvalidEmail(_)) => (),
            _ => panic!("Expected InvalidEmail error"),
        }
    }

    #[tokio::test]
    async fn test_send_confirmation_valid_email() {
        let reservation = models::ConfirmedReservation {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_name: "John Doe".to_string(),
            user_email: "john@example.com".to_string(),
            reservation_token: "token789".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            status: models::Confirmed { verified_at: OffsetDateTime::now_utc() },
        };

        let result = send_confirmation("john@example.com", &reservation).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_confirmation_invalid_email() {
        let reservation = models::ConfirmedReservation {
            id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            user_name: "John Doe".to_string(),
            user_email: "john@example.com".to_string(),
            reservation_token: "token789".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            status: models::Confirmed { verified_at: OffsetDateTime::now_utc() },
        };

        let result = send_confirmation("invalid-email", &reservation).await;
        assert!(result.is_err());
        match result {
            Err(EmailError::InvalidEmail(_)) => (),
            _ => panic!("Expected InvalidEmail error"),
        }
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user@domain.co.uk"));
        assert!(!is_valid_email("invalid-email"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email(""));
    }
}
