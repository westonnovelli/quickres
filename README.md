# Quick Reservations

A simple event reservation system built with Rust, Axum, and SQLite. This application provides a REST API for managing events and reservations with email verification.

## Features

- **Event Management**: View event details and availability
- **Reservation System**: Create reservations with email verification
- **Email Notifications**: Send verification and confirmation emails
- **Token-based Security**: Secure verification and magic link tokens
- **SQLite Database**: Lightweight, file-based database storage
- **Environment Configuration**: Flexible configuration via `.env` file

## Quick Start

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd quick-res
   ```

2. **Copy and configure environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Run the application**
   ```bash
   cargo run
   ```

4. **Access the API**
   - Base URL: `http://localhost:8000`
   - Health check: `GET /`

## Environment Configuration

The application uses the [`dotenvy`](https://crates.io/crates/dotenvy) crate to load environment variables from a `.env` file. All configuration is centralized in the `src/config.rs` module.

### Core Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:quick-res.db` | Database connection string |
| `BASE_URL` | `http://localhost:8000` | Base URL for generating links in emails |
| `APP_NAME` | `Quick Reservations` | Application name used in emails and responses |
| `APP_ENVIRONMENT` | `development` | Environment: `development`, `staging`, or `production` |
| `PORT` | `8000` | Server port (Note: Shuttle may override this) |

### Email Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `EMAIL_FROM` | `noreply@quick-res.example.com` | From email address |
| `EMAIL_FROM_NAME` | `Quick Reservations` | From email display name |
| `EMAIL_PROVIDER` | `console` | Email provider: `console`, `smtp`, `sendgrid`, `mailgun`, `ses` |

#### SMTP Configuration (when `EMAIL_PROVIDER=smtp`)

| Variable | Default | Description |
|----------|---------|-------------|
| `SMTP_HOST` | `localhost` | SMTP server hostname |
| `SMTP_PORT` | `587` | SMTP server port |
| `SMTP_USERNAME` | - | SMTP username (optional) |
| `SMTP_PASSWORD` | - | SMTP password (optional) |
| `SMTP_TLS` | `true` | Enable TLS encryption |

#### SendGrid Configuration (when `EMAIL_PROVIDER=sendgrid`)

| Variable | Default | Description |
|----------|---------|-------------|
| `SENDGRID_API_KEY` | - | SendGrid API key |

### Security Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | `your-jwt-secret-key-change-this-in-production` | JWT signing secret (change in production!) |
| `SESSION_SECRET` | `your-session-secret-change-this-in-production` | Session signing secret (change in production!) |

### Rate Limiting

| Variable | Default | Description |
|----------|---------|-------------|
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | `60` | Maximum requests per minute per IP |
| `RATE_LIMIT_BURST` | `10` | Burst allowance for rate limiting |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_LEVEL` | `info` | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `LOG_FORMAT` | `json` | Log format: `json` or `pretty` |

### CORS Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CORS_ALLOWED_ORIGINS` | `http://localhost:3000,http://localhost:8080` | Comma-separated list of allowed origins |
| `CORS_ALLOWED_METHODS` | `GET,POST,PUT,DELETE,OPTIONS` | Comma-separated list of allowed HTTP methods |
| `CORS_ALLOWED_HEADERS` | `Content-Type,Authorization,X-Requested-With` | Comma-separated list of allowed headers |

## API Endpoints

### Events

- **GET /events/{id}** - Get event details
  - Returns event information including capacity and timing
  - Response: `200 OK` with event JSON

### Reservations

- **POST /reservations** - Create a new reservation
  - Request body: `{ "event_id": "uuid", "user_name": "string", "user_email": "email" }`
  - Creates a pending reservation and sends verification email
  - Response: `201 Created` with reservation details

- **GET /verify/{token}** - Verify reservation
  - Confirms a pending reservation using the verification token
  - Sends confirmation email with magic link
  - Response: `200 OK` with confirmation details

- **GET /reservation/{magic_token}** - Access reservation details
  - View confirmed reservation details using magic link token
  - Only works for confirmed reservations
  - Response: `200 OK` with reservation JSON

## Database Schema

The application uses SQLite with the following tables:

### Events Table
```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    capacity INTEGER NOT NULL,
    location TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Reservations Table
```sql
CREATE TABLE reservations (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    reservation_token TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES events (id)
);
```

## Development

### Prerequisites

- Rust 1.70+ 
- Cargo

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test config::tests
```

### Project Structure

```
src/
├── main.rs          # Application entry point and route handlers
├── config.rs        # Environment configuration management
├── db.rs           # Database operations and models
├── email.rs        # Email sending functionality
├── error.rs        # Error handling and types
└── models.rs       # Request/response models and validation
```

### Email Testing

By default, the application uses `EMAIL_PROVIDER=console`, which prints emails to stdout instead of sending them. This is perfect for development and testing.

To test real email sending:

1. **SMTP (e.g., Gmail)**:
   ```env
   EMAIL_PROVIDER=smtp
   SMTP_HOST=smtp.gmail.com
   SMTP_PORT=587
   SMTP_USERNAME=your-email@gmail.com
   SMTP_PASSWORD=your-app-password
   SMTP_TLS=true
   ```

2. **SendGrid**:
   ```env
   EMAIL_PROVIDER=sendgrid
   SENDGRID_API_KEY=your-api-key
   ```

## Deployment

### Environment Variables for Production

When deploying to production, ensure you:

1. **Change default secrets**:
   ```env
   JWT_SECRET=your-strong-random-secret-here
   SESSION_SECRET=your-strong-random-secret-here
   ```

2. **Set production environment**:
   ```env
   APP_ENVIRONMENT=production
   BASE_URL=https://your-domain.com
   ```

3. **Configure email provider**:
   ```env
   EMAIL_PROVIDER=sendgrid  # or smtp
   EMAIL_FROM=noreply@your-domain.com
   EMAIL_FROM_NAME=Your App Name
   ```

4. **Set appropriate CORS**:
   ```env
   CORS_ALLOWED_ORIGINS=https://your-frontend.com
   ```

### Security Considerations

- Always use HTTPS in production
- Change default JWT and session secrets
- Use environment-specific database URLs
- Configure appropriate CORS origins
- Set up proper logging and monitoring
- Consider rate limiting configuration based on your needs

### Shuttle Deployment

This application is configured for [Shuttle](https://shuttle.rs) deployment:

```bash
# Deploy to Shuttle
shuttle deploy
```

Note: When using Shuttle, the `PORT` environment variable may be overridden by the platform.

## License

[Add your license information here]

## Contributing

[Add contributing guidelines here]

## Support

[Add support contact information here]
