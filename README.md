# FeedTape Backend

Minimal backend API for FeedTape RSS to Audio service built with Rust, Axum, PostgreSQL, and AWS Polly.

## 🚀 Features

- **User Management** - Profile, settings, and subscription management
- **Feed Management** - CRUD operations for RSS feed URLs
- **Text-to-Speech** - Convert text to audio using AWS Polly with 6 language support
- **Authentication** - JWT-based auth with refresh tokens (OAuth ready)
- **Usage Tracking** - Daily quota enforcement and usage statistics
- **Free Trial** - 7-day trial with 20,000 characters/day (20 minutes)
- **Pro Tier** - 200,000 characters/day (200 minutes) with neural voices

## 📋 Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Docker & Docker Compose (for PostgreSQL)
- AWS Account with Polly access
- direnv (optional, for environment management)

## 🛠️ Tech Stack

- **Framework:** Axum 0.7
- **Database:** PostgreSQL 15
- **ORM:** SQLx (compile-time query checking)
- **TTS:** AWS Polly
- **Language Detection:** Lingua-rs
- **Authentication:** JWT (jsonwebtoken)
- **Logging:** Tracing with structured logs

## 📦 Quick Start

### 1. Clone and Setup Environment

```bash
# Clone the repository
cd feedtape-backend

# Copy environment template
cp .env.example .env

# Edit .env with your credentials
# Required: DATABASE_URL, JWT_SECRET, AWS credentials
```

### 2. Start PostgreSQL

```bash
docker-compose up -d
```

### 3. Run the Server

```bash
# Development mode with auto-reload
cargo run

# Or with cargo-watch for auto-reload on changes
cargo watch -x run
```

The server will start on `http://localhost:8080`

## 📚 API Endpoints

### Health Checks
- `GET /health` - Simple health check
- `GET /health/ready` - Readiness check with database status

### Authentication
- `POST /auth/refresh` - Refresh access token
- `POST /auth/logout` - Logout (revoke refresh token)
- `POST /auth/logout/all` - Logout from all devices (requires auth)

### User Management
- `GET /api/me` - Get user profile with settings and subscription
- `PATCH /api/me` - Update user settings

### Feed Management
- `GET /api/feeds` - List user's feeds
- `POST /api/feeds` - Create new feed
- `PUT /api/feeds/:feedId` - Update feed title
- `DELETE /api/feeds/:feedId` - Delete feed

### Text-to-Speech
- `POST /api/tts/synthesize` - Convert text to speech
- `GET /api/tts/usage` - Get usage statistics and history

## 🔐 Environment Variables

### Required
```bash
DATABASE_URL=postgresql://feedtape:password@localhost:5432/feedtape
JWT_SECRET=your-super-secret-jwt-key-change-this
AWS_ACCESS_KEY_ID=your-aws-access-key
AWS_SECRET_ACCESS_KEY=your-aws-secret-key
```

### Optional (with defaults)
```bash
HOST=0.0.0.0
PORT=8080
AWS_REGION=us-east-1
JWT_EXPIRATION_HOURS=1
REFRESH_TOKEN_EXPIRATION_DAYS=30
RUST_LOG=debug
LOG_FORMAT=pretty  # or 'json' for production
ENVIRONMENT=development  # or 'production'
```

## 🗄️ Database Schema

The application uses 4 main tables:
- `users` - User accounts with OAuth and subscription info
- `feeds` - RSS feed URLs per user
- `refresh_tokens` - JWT refresh token storage
- `usage_tracking` - Daily TTS usage statistics

Schema is automatically created when starting PostgreSQL with Docker Compose.

## 🌍 Supported Languages

- English (en) - Voice: Joanna
- Spanish (es) - Voice: Lucia
- French (fr) - Voice: Lea
- German (de) - Voice: Vicki
- Italian (it) - Voice: Bianca
- Portuguese (pt) - Voice: Ines

All voices use AWS Polly Neural engine for high quality.

## 📊 Usage Limits

### Free Tier (7-day trial)
- 20,000 characters/day (~20 minutes)
- Maximum 3 feeds
- Standard voice quality only

### Pro Tier
- 200,000 characters/day (~200 minutes)
- Unlimited feeds
- Neural voice quality

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Check code
cargo check

# Run clippy for lints
cargo clippy
```

## 📝 Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── db.rs                # Database connection pool
├── error.rs             # Error types and handling
├── handlers/            # HTTP request handlers
│   ├── auth.rs
│   ├── feed.rs
│   ├── health.rs
│   ├── tts.rs
│   └── user.rs
├── services/            # Business logic
│   ├── auth_service.rs
│   ├── feed_service.rs
│   ├── tts_service.rs
│   └── user_service.rs
├── repositories/        # Database access
│   ├── feed_repository.rs
│   ├── refresh_token_repository.rs
│   ├── usage_repository.rs
│   └── user_repository.rs
├── models/              # Domain models
│   ├── feed.rs
│   └── user.rs
├── dto/                 # API request/response types
│   ├── auth.rs
│   ├── error.rs
│   ├── feed.rs
│   ├── tts.rs
│   ├── usage.rs
│   └── user.rs
├── middleware/          # HTTP middleware
│   ├── auth.rs          # JWT authentication
│   └── request_id.rs    # Request tracking
└── utils/               # Utilities
    ├── jwt.rs           # JWT token management
    └── language.rs      # Language detection
```

## 🚢 Deployment

### Railway.app (Recommended)

1. Create new project on Railway
2. Add PostgreSQL database
3. Set environment variables
4. Connect GitHub repository
5. Railway will auto-deploy on push

### Docker

```bash
# Build image
docker build -t feedtape-backend .

# Run container
docker run -p 8080:8080 --env-file .env feedtape-backend
```

## 🔍 Development

### Enable direnv (optional)

```bash
# Allow direnv
direnv allow

# Environment variables will auto-load from .env
```

### Watch mode

```bash
# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

### Database Migrations

Currently using a single `schema.sql` file for simplicity. For production, consider using a migration tool like:
- `sqlx migrate`
- `diesel`

## 📖 API Documentation

Full API specification available in `openapi.yaml`. View with:
- Swagger Editor: https://editor.swagger.io/
- Import the file for interactive API testing

## 🤝 Contributing

This is an MVP project. Future enhancements:
- OAuth provider implementations (Apple, Google, GitHub)
- Subscription receipt validation
- Rate limiting
- Caching layer (Redis)
- WebSocket support
- Background jobs

## 📄 License

Private project - All rights reserved

## 🔗 Related Projects

- [FeedTape Mobile App](../feedtape-app) - React Native app
- [Project Context](./spec/context/PROJECT_CONTEXT.md) - Full product vision
- [Implementation Log](./spec/mvp/log.md) - Development decisions

## 📞 Support

For issues or questions, contact: support@feedtape.app

---

**Built with ❤️ using Rust and Axum**
