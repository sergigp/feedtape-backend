# FeedTape Backend - Local Development Setup

## Prerequisites

- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- Docker Desktop (for PostgreSQL)
- AWS Account (for Polly TTS)

## Step-by-Step Setup

### 1. Clone and Install Dependencies

```bash
git clone <repo-url>
cd feedtape-backend
```

### 2. Configure Environment Variables

```bash
# Copy the example file
cp .env.example .env

# Edit .env with your real secrets
nano .env  # or use your preferred editor
```

**Required secrets to add:**

```bash
# Generate a secure JWT secret (run this command):
openssl rand -base64 32

# Add to .env:
JWT_SECRET=<paste-the-output-here>

# Add your AWS credentials from AWS Console:
AWS_ACCESS_KEY_ID=<your-key>
AWS_SECRET_ACCESS_KEY=<your-secret>
```

### 3. Start PostgreSQL

```bash
docker-compose up -d
```

### 4. Run the Backend

```bash
cargo run
```

The server will start on http://localhost:8080

## Environment Variables Guide

### Safe Defaults (Already in .env.example)
These work out-of-the-box for local development:
- `DATABASE_URL` - Local PostgreSQL connection
- `HOST`, `PORT` - Server configuration
- `AWS_REGION` - AWS region (us-east-1)

### Secrets You Must Provide
These **must** be added to your `.env` file:

#### JWT Secret
```bash
# Generate with:
openssl rand -base64 32

# Add to .env:
JWT_SECRET=<generated-secret>
```

#### AWS Credentials
Get from AWS Console → IAM → Users → Security Credentials:
```bash
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/...
```

#### OAuth Credentials (Optional for MVP)
For Apple Sign In:
```bash
APPLE_CLIENT_ID=com.yourapp.service
APPLE_TEAM_ID=TEAM123456
APPLE_KEY_ID=KEY123456
```

## Security Notes

⚠️ **Never commit `.env` to Git!**
- `.env` is in `.gitignore` - keep it that way
- `.env.example` is for documentation only
- Use different secrets for production

## Troubleshooting

### "Missing DATABASE_URL"
Make sure `.env` exists and has `DATABASE_URL` set.

### "AWS credentials not found"
1. Check `.env` has AWS credentials
2. Verify credentials are valid in AWS Console
3. Ensure the IAM user has Polly permissions

### "Docker container won't start"
```bash
docker-compose down
docker volume rm feedtape-backend_postgres_data
docker-compose up -d
```

## Optional: direnv for Auto-loading

Install [direnv](https://direnv.net/):
```bash
brew install direnv  # macOS
```

The `.envrc` file will auto-load `.env` when you `cd` into the project:
```bash
direnv allow
```
