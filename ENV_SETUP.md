# Environment Setup Guide

This guide explains how to set up environment variables for all ConHub microservices.

## Overview

Each microservice in ConHub is **highly decoupled** and **independent**. Each service folder contains:
- Its own `.env.example` file with all required environment variables
- Its own dependencies and configuration
- Complete independence - can be moved to a separate repository and still work

## Quick Setup

### 1. Generate JWT Keys

First, generate RSA key pair for JWT authentication:

```powershell
.\generate-jwt-keys.ps1
```

If you don't have OpenSSL installed, the script will provide alternatives.

### 2. Set Up Environment Files

Run the setup script to create `.env` files for all services:

```powershell
.\setup-env.ps1
```

This will:
- Copy `.env.example` to `.env` for each microservice
- Populate JWT keys automatically
- Create `frontend/.env.local`

### 3. Update Configuration

Review and update the `.env` files in each service folder with your actual values:

- **Database credentials** (PostgreSQL)
- **Redis URL**
- **API keys** (OpenAI, Stripe, GitHub, etc.)
- **OAuth credentials** (GitHub, Google, Bitbucket)
- **Service URLs** (if different from defaults)

## Microservices

### Auth Service (`auth/`)
**Port:** 3010  
**Dependencies:** PostgreSQL, Redis  
**Key Variables:**
- `JWT_PRIVATE_KEY` - RSA private key for signing JWTs
- `JWT_PUBLIC_KEY` - RSA public key for verifying JWTs
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string

### Billing Service (`billing/`)
**Port:** 3011  
**Dependencies:** PostgreSQL, Redis  
**Key Variables:**
- `JWT_PUBLIC_KEY` - For verifying JWTs
- `DATABASE_URL` - PostgreSQL connection string
- `STRIPE_SECRET_KEY` - Stripe API secret key
- `STRIPE_WEBHOOK_SECRET` - Stripe webhook secret

### Client/AI Service (`client/`)
**Port:** 3012  
**Dependencies:** PostgreSQL, Redis, Qdrant  
**Key Variables:**
- `JWT_PUBLIC_KEY` - For verifying JWTs
- `DATABASE_URL` - PostgreSQL connection string
- `QDRANT_URL` - Qdrant vector database URL
- `OPENAI_API_KEY` - OpenAI API key
- `GITHUB_ACCESS_TOKEN` - GitHub personal access token

### Data Service (`data/`)
**Port:** 3013  
**Dependencies:** PostgreSQL, Redis, Qdrant  
**Key Variables:**
- `JWT_PUBLIC_KEY` - For verifying JWTs
- `DATABASE_URL` - PostgreSQL connection string
- `QDRANT_URL` - Qdrant vector database URL
- `GITHUB_ACCESS_TOKEN` - GitHub token
- `GOOGLE_DRIVE_CLIENT_ID` - Google Drive OAuth
- `NOTION_API_KEY` - Notion integration key

### Security Service (`security/`)
**Port:** 3014  
**Dependencies:** PostgreSQL, Redis  
**Key Variables:**
- `JWT_PUBLIC_KEY` - For verifying JWTs
- `DATABASE_URL` - PostgreSQL connection string
- `ENCRYPTION_KEY` - For encrypting sensitive data

### Webhook Service (`webhook/`)
**Port:** 3015  
**Dependencies:** PostgreSQL  
**Key Variables:**
- `JWT_PUBLIC_KEY` - For verifying JWTs
- `DATABASE_URL` - PostgreSQL connection string
- `GITHUB_WEBHOOK_SECRET` - GitHub webhook secret
- `STRIPE_WEBHOOK_SECRET` - Stripe webhook secret

### Frontend (`frontend/`)
**Port:** 3000  
**Dependencies:** None (connects to backend services)  
**Key Variables:**
- `NEXTAUTH_SECRET` - NextAuth.js secret
- `GITHUB_CLIENT_ID` - GitHub OAuth
- `GOOGLE_CLIENT_ID` - Google OAuth
- `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY` - Stripe public key

## Required Infrastructure

### PostgreSQL Database
```bash
# Using Docker
docker run -d \
  --name conhub-postgres \
  -e POSTGRES_DB=conhub \
  -e POSTGRES_USER=conhub \
  -e POSTGRES_PASSWORD=conhub_password \
  -p 5432:5432 \
  postgres:15-alpine
```

### Redis Cache
```bash
# Using Docker
docker run -d \
  --name conhub-redis \
  -p 6379:6379 \
  redis:7-alpine
```

### Qdrant Vector Database (for AI/Data services)
```bash
# Using Docker
docker run -d \
  --name conhub-qdrant \
  -p 6333:6333 \
  -p 6334:6334 \
  qdrant/qdrant:latest
```

## Environment Modes

Services support two modes via `ENV_MODE`:

- **`local`** - Services run locally, connect to `localhost` databases
- **`docker`** - Services run in Docker, connect to service names

The mode is automatically set:
- `npm start` → `ENV_MODE=local`
- `docker-compose up` → `ENV_MODE=docker`

## Service Independence

Each microservice is designed to be **completely independent**:

1. **Self-contained configuration** - All env vars in service folder
2. **Graceful degradation** - Services continue if dependencies are unavailable
3. **No shared state** - Each service has its own database schema
4. **Portable** - Can move service folder to another repo and it works

### Example: Running a Single Service

```bash
# Navigate to service folder
cd auth

# Copy and configure .env
cp .env.example .env
# Edit .env with your values

# Run the service
cargo run
```

The service will:
- Load its own `.env` file
- Connect to its dependencies
- Start on its configured port
- Work independently of other services

## Security Best Practices

1. **Never commit `.env` files** - They're in `.gitignore`
2. **Rotate JWT keys regularly** - Regenerate with `generate-jwt-keys.ps1`
3. **Use strong secrets** - Generate with `openssl rand -base64 32`
4. **Limit key access** - Only auth service needs private key
5. **Use environment-specific keys** - Different keys for dev/staging/prod

## Troubleshooting

### "No such host is known" error
- Check `DATABASE_URL` is correct
- Ensure PostgreSQL is running
- Try `localhost` instead of service name in local mode

### "No public key found" error
- Run `generate-jwt-keys.ps1` to create keys
- Run `setup-env.ps1` to populate .env files
- Check `JWT_PUBLIC_KEY` is set in service's `.env`

### Service won't start
- Check all required env vars are set
- Verify dependencies (PostgreSQL, Redis) are running
- Check port is not already in use
- Review service logs for specific errors

## Additional Resources

- [Main README](./README.md) - Project overview
- [Docker Setup](./docker-compose.yml) - Container orchestration
- [Feature Toggles](./feature-toggles.json) - Feature flags configuration
