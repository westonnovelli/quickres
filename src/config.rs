use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub base_url: String,
    pub app_name: String,
    pub app_environment: String,
    pub port: u16,
    pub email_from: String,
    pub email_from_name: String,
    pub email_provider: String,
    pub jwt_secret: String,
    pub session_secret: String,
    pub rate_limit_requests_per_minute: u32,
    pub rate_limit_burst: u32,
    pub log_level: String,
    pub log_format: String,
    pub cors_allowed_origins: Vec<String>,
    pub cors_allowed_methods: Vec<String>,
    pub cors_allowed_headers: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:quick-res.db".to_string()),
            base_url: env::var("BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            app_name: env::var("APP_NAME")
                .unwrap_or_else(|_| "Quick Reservations".to_string()),
            app_environment: env::var("APP_ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            email_from: env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "noreply@quick-res.example.com".to_string()),
            email_from_name: env::var("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| "Quick Reservations".to_string()),
            email_provider: env::var("EMAIL_PROVIDER")
                .unwrap_or_else(|_| "console".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-jwt-secret-key-change-this-in-production".to_string()),
            session_secret: env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "your-session-secret-change-this-in-production".to_string()),
            rate_limit_requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            rate_limit_burst: env::var("RATE_LIMIT_BURST")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            log_format: env::var("LOG_FORMAT")
                .unwrap_or_else(|_| "json".to_string()),
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            cors_allowed_methods: env::var("CORS_ALLOWED_METHODS")
                .unwrap_or_else(|_| "GET,POST,PUT,DELETE,OPTIONS".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            cors_allowed_headers: env::var("CORS_ALLOWED_HEADERS")
                .unwrap_or_else(|_| "Content-Type,Authorization,X-Requested-With".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        })
    }

    /// Get SMTP configuration if email provider is set to SMTP
    pub fn smtp_config(&self) -> Option<SmtpConfig> {
        if self.email_provider.to_lowercase() == "smtp" {
            Some(SmtpConfig {
                host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
                port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .unwrap_or(587),
                username: env::var("SMTP_USERNAME").ok(),
                password: env::var("SMTP_PASSWORD").ok(),
                tls: env::var("SMTP_TLS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            })
        } else {
            None
        }
    }

    /// Get SendGrid API key if email provider is set to SendGrid
    pub fn sendgrid_api_key(&self) -> Option<String> {
        if self.email_provider.to_lowercase() == "sendgrid" {
            env::var("SENDGRID_API_KEY").ok()
        } else {
            None
        }
    }

    /// Check if the application is running in production
    pub fn is_production(&self) -> bool {
        self.app_environment.to_lowercase() == "production"
    }

    /// Check if the application is running in development
    pub fn is_development(&self) -> bool {
        self.app_environment.to_lowercase() == "development"
    }
}

/// SMTP configuration for email sending
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_with_defaults() {
        // Clear environment variables to test defaults
        env::remove_var("DATABASE_URL");
        env::remove_var("BASE_URL");
        env::remove_var("APP_NAME");
        env::remove_var("PORT");
        env::remove_var("EMAIL_PROVIDER");
        env::remove_var("APP_ENVIRONMENT");
        env::remove_var("EMAIL_FROM");
        env::remove_var("EMAIL_FROM_NAME");
        env::remove_var("JWT_SECRET");
        env::remove_var("SESSION_SECRET");
        env::remove_var("RATE_LIMIT_REQUESTS_PER_MINUTE");
        env::remove_var("RATE_LIMIT_BURST");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_FORMAT");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("CORS_ALLOWED_METHODS");
        env::remove_var("CORS_ALLOWED_HEADERS");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.database_url, "sqlite:quick-res.db");
        assert_eq!(config.base_url, "http://localhost:8000");
        assert_eq!(config.app_name, "Quick Reservations");
        assert_eq!(config.port, 8000);
        assert_eq!(config.email_provider, "console");
    }

    #[test]
    fn test_config_from_env_with_custom_values() {
        // Clear all environment variables first
        env::remove_var("DATABASE_URL");
        env::remove_var("BASE_URL");
        env::remove_var("APP_NAME");
        env::remove_var("PORT");
        env::remove_var("EMAIL_PROVIDER");
        env::remove_var("APP_ENVIRONMENT");
        env::remove_var("EMAIL_FROM");
        env::remove_var("EMAIL_FROM_NAME");
        env::remove_var("JWT_SECRET");
        env::remove_var("SESSION_SECRET");
        env::remove_var("RATE_LIMIT_REQUESTS_PER_MINUTE");
        env::remove_var("RATE_LIMIT_BURST");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_FORMAT");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("CORS_ALLOWED_METHODS");
        env::remove_var("CORS_ALLOWED_HEADERS");
        
        // Set custom values
        env::set_var("DATABASE_URL", "sqlite:test.db");
        env::set_var("BASE_URL", "https://example.com");
        env::set_var("APP_NAME", "Test App");
        env::set_var("PORT", "3000");
        
        let config = Config::from_env().unwrap();
        
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.base_url, "https://example.com");
        assert_eq!(config.app_name, "Test App");
        assert_eq!(config.port, 3000);
        
        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("BASE_URL");
        env::remove_var("APP_NAME");
        env::remove_var("PORT");
    }

    #[test]
    fn test_smtp_config() {
        // Clear all environment variables first
        env::remove_var("DATABASE_URL");
        env::remove_var("BASE_URL");
        env::remove_var("APP_NAME");
        env::remove_var("PORT");
        env::remove_var("EMAIL_PROVIDER");
        env::remove_var("APP_ENVIRONMENT");
        env::remove_var("EMAIL_FROM");
        env::remove_var("EMAIL_FROM_NAME");
        env::remove_var("JWT_SECRET");
        env::remove_var("SESSION_SECRET");
        env::remove_var("RATE_LIMIT_REQUESTS_PER_MINUTE");
        env::remove_var("RATE_LIMIT_BURST");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_FORMAT");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("CORS_ALLOWED_METHODS");
        env::remove_var("CORS_ALLOWED_HEADERS");
        env::remove_var("SMTP_HOST");
        env::remove_var("SMTP_PORT");
        env::remove_var("SMTP_USERNAME");
        env::remove_var("SMTP_PASSWORD");
        env::remove_var("SMTP_TLS");
        
        env::set_var("EMAIL_PROVIDER", "smtp");
        env::set_var("SMTP_HOST", "smtp.example.com");
        env::set_var("SMTP_PORT", "465");
        env::set_var("SMTP_USERNAME", "user@example.com");
        env::set_var("SMTP_PASSWORD", "password123");
        
        let config = Config::from_env().unwrap();
        let smtp_config = config.smtp_config().unwrap();
        
        assert_eq!(smtp_config.host, "smtp.example.com");
        assert_eq!(smtp_config.port, 465);
        assert_eq!(smtp_config.username, Some("user@example.com".to_string()));
        assert_eq!(smtp_config.password, Some("password123".to_string()));
        
        // Clean up
        env::remove_var("EMAIL_PROVIDER");
        env::remove_var("SMTP_HOST");
        env::remove_var("SMTP_PORT");
        env::remove_var("SMTP_USERNAME");
        env::remove_var("SMTP_PASSWORD");
    }

    #[test]
    fn test_environment_detection() {
        // Clear all environment variables first
        env::remove_var("DATABASE_URL");
        env::remove_var("BASE_URL");
        env::remove_var("APP_NAME");
        env::remove_var("PORT");
        env::remove_var("EMAIL_PROVIDER");
        env::remove_var("APP_ENVIRONMENT");
        env::remove_var("EMAIL_FROM");
        env::remove_var("EMAIL_FROM_NAME");
        env::remove_var("JWT_SECRET");
        env::remove_var("SESSION_SECRET");
        env::remove_var("RATE_LIMIT_REQUESTS_PER_MINUTE");
        env::remove_var("RATE_LIMIT_BURST");
        env::remove_var("LOG_LEVEL");
        env::remove_var("LOG_FORMAT");
        env::remove_var("CORS_ALLOWED_ORIGINS");
        env::remove_var("CORS_ALLOWED_METHODS");
        env::remove_var("CORS_ALLOWED_HEADERS");
        
        env::set_var("APP_ENVIRONMENT", "production");
        let config = Config::from_env().unwrap();
        assert!(config.is_production());
        assert!(!config.is_development());
        
        env::set_var("APP_ENVIRONMENT", "development");
        let config = Config::from_env().unwrap();
        assert!(!config.is_production());
        assert!(config.is_development());
        
        env::remove_var("APP_ENVIRONMENT");
    }
}
