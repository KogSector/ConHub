# Auth0 Setup Guide for ConHub

(Originally `docs/AUTH0_SETUP.md`)

## Quick Start (30 minutes)

If you just want to get Auth0 working end‑to‑end as fast as possible, follow this high‑level flow. For full details, see the sections below.

1. **Create Auth0 tenant & API**  
   - Create a tenant (e.g. `conhub-dev`) in the Auth0 dashboard.  
   - Note your domain, e.g. `conhub-dev.us.auth0.com` → this is `AUTH0_DOMAIN`.  
   - Create an API with identifier `https://api.conhub.dev` and algorithm `RS256` → this is `AUTH0_AUDIENCE`.

2. **Create SPA application**  
   - Create a **Single Page Web Application** called `ConHub Web App`.  
   - Configure URIs:  
     - **Allowed Callback URLs**: `http://localhost:3000/auth/callback`  
     - **Allowed Logout URLs**: `http://localhost:3000/`  
     - **Allowed Web Origins**: `http://localhost:3000`  
   - Copy the **Client ID** → this is `NEXT_PUBLIC_AUTH0_CLIENT_ID`.

3. **Generate JWT keys** (for ConHub internal JWTs)  
```bash
openssl genrsa -out private.pem 2048
openssl rsa -in private.pem -pubout -out public.pem
```
Copy the contents of `private.pem` and `public.pem` into `JWT_PRIVATE_KEY` and `JWT_PUBLIC_KEY` in the auth service `.env`.

4. **Configure auth service env** (`auth/.env`)  
- In `auth/.env`, set at minimum:  
  - `AUTH0_DOMAIN`  
  - `AUTH0_AUDIENCE`  
  - `JWT_PRIVATE_KEY` / `JWT_PUBLIC_KEY`  
  - `DATABASE_URL_NEON`  
  - `REDIS_URL`  
  - `AUTH_SERVICE_PORT=3010`.

5. **Configure frontend env** (`frontend/.env.local`)  
- In `frontend/.env.local`, set:  
  - `NEXT_PUBLIC_AUTH0_DOMAIN`  
  - `NEXT_PUBLIC_AUTH0_CLIENT_ID`  
  - `NEXT_PUBLIC_AUTH0_AUDIENCE`  
  - `NEXT_PUBLIC_AUTH0_REDIRECT_URI=http://localhost:3000/auth/callback`  
  - `NEXT_PUBLIC_AUTH0_LOGOUT_REDIRECT_URI=http://localhost:3000/`  
  - `NEXT_PUBLIC_AUTH_SERVICE_URL=http://localhost:3010`.

6. **Run database migration**  
```bash
cd database
sqlx migrate run --database-url "$DATABASE_URL_NEON"
```

7. **Start services**  
```bash
# Terminal 1
cd auth && cargo run

# Terminal 2
cd frontend && npm install && npm run dev
```

8. **Test sign‑in**  
- Open `http://localhost:3000`.  
- Click **Sign In** → you should be redirected to Auth0, complete login, then land back in the app.

---

This guide walks you through setting up Auth0 authentication for ConHub from scratch.

## Table of Contents
1. [Auth0 Tenant Setup](#auth0-tenant-setup)
2. [Create API](#create-api)
3. [Create SPA Application](#create-spa-application)
4. [Configure Social Connections](#configure-social-connections)
5. [Environment Variables](#environment-variables)
6. [Database Migration](#database-migration)
7. [Testing](#testing)

---

## 1. Auth0 Tenant Setup

1. Go to [auth0.com](https://auth0.com) and sign up or log in
2. Create a new tenant (or use existing):
   - Choose a tenant name (e.g., `conhub-dev`)
   - Select region (e.g., `US`, `EU`, `AU`)
   - Your domain will be: `{tenant-name}.{region}.auth0.com`
   - **Save this domain** - you'll need it as `AUTH0_DOMAIN`

Example: `conhub-dev.us.auth0.com`

---

## 2. Create API

This represents your ConHub backend that Auth0 will issue tokens for.

1. In Auth0 Dashboard, go to **Applications → APIs**
2. Click **Create API**
3. Fill in:
   - **Name**: `ConHub API`
   - **Identifier**: `https://api.conhub.dev` (can be any URI, doesn't need to be real)
   - **Signing Algorithm**: `RS256`
4. Click **Create**
5. **Save the Identifier** - this is your `AUTH0_AUDIENCE`

### API Settings (Optional)
- **Token Expiration**: 86400 seconds (24 hours) - adjust as needed
- **Allow Offline Access**: Enable if you want refresh tokens
- **RBAC Settings**: Can configure later for role-based access

---

## 3. Create SPA Application

This represents your frontend application.

1. In Auth0 Dashboard, go to **Applications → Applications**
2. Click **Create Application**
3. Fill in:
   - **Name**: `ConHub Web App`
   - **Application Type**: Select **Single Page Web Applications**
4. Click **Create**

### Configure Application Settings

Go to the **Settings** tab:

#### Basic Information
- **Client ID**: Copy this - you'll need it as `NEXT_PUBLIC_AUTH0_CLIENT_ID`
- **Client Secret**: Not needed for SPA (keep it secret if shown)

#### Application URIs

**Allowed Callback URLs** (comma-separated):
```
http://localhost:3000/auth/callback,
https://your-production-domain.com/auth/callback
```

**Allowed Logout URLs**:
```
http://localhost:3000/,
https://your-production-domain.com/
```

**Allowed Web Origins**:
```
http://localhost:3000,
https://your-production-domain.com
```

**Allowed Origins (CORS)**:
```
http://localhost:3000,
https://your-production-domain.com
```

#### Advanced Settings

Go to **Advanced Settings → OAuth**:
- **JsonWebToken Signature Algorithm**: `RS256`
- **OIDC Conformant**: Enabled (should be default)

Go to **Advanced Settings → Grant Types**:
- Ensure these are checked:
  - ✅ Authorization Code
  - ✅ Refresh Token
  - ✅ Implicit (optional, not recommended)

#### APIs

- Under **APIs** tab, authorize your SPA to request tokens for `ConHub API`

**Save Changes**

---

## 4. Configure Social Connections

Enable social login providers (Google, GitHub, Bitbucket).

### Enable Database Connection (Email/Password)

1. Go to **Authentication → Database**
2. The default `Username-Password-Authentication` should be enabled
3. Configure password policy as needed
4. Enable **Signup** if you want users to self-register

### Enable Google

1. Go to **Authentication → Social**
2. Find **Google** and click the toggle or **+** icon
3. You'll need Google OAuth credentials:

#### Get Google Credentials:
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a project or select existing
3. Go to **APIs & Services → Credentials**
4. Click **Create Credentials → OAuth 2.0 Client ID**
5. Application type: **Web application**
6. **Authorized redirect URIs**: Add Auth0's callback
   ```
   https://{your-auth0-domain}/login/callback
   ```
   Example: `https://conhub-dev.us.auth0.com/login/callback`
7. Copy **Client ID** and **Client Secret**

#### Configure in Auth0:
- Paste Google **Client ID**
- Paste Google **Client Secret**
- **Attributes**: Select `email`, `profile`
- **Permissions**: `email`, `profile`
- Click **Save**

### Enable GitHub

1. In Auth0 Dashboard, go to **Authentication → Social**
2. Find **GitHub** and enable it
3. You'll need GitHub OAuth App credentials:

#### Get GitHub Credentials:
1. Go to [GitHub Settings → Developer settings → OAuth Apps](https://github.com/settings/developers)
2. Click **New OAuth App**
3. Fill in:
   - **Application name**: `ConHub`
   - **Homepage URL**: `http://localhost:3000` (or your domain)
   - **Authorization callback URL**: Auth0's callback
     ```
     https://{your-auth0-domain}/login/callback
     ```
4. Click **Register application**
5. Copy **Client ID**
6. Generate a **Client Secret** and copy it

#### Configure in Auth0:
- Paste GitHub **Client ID**
- Paste GitHub **Client Secret**
- Click **Save**

### Enable Bitbucket (Optional)

1. In Auth0 Dashboard, go to **Authentication → Social**
2. Bitbucket might not be in the default list - you may need to use a custom OAuth connection
3. Alternatively, use Auth0's **Social Connections → Create Connection → OAuth2**

#### Get Bitbucket Credentials:
1. Go to [Bitbucket Settings → OAuth consumers](https://bitbucket.org/account/settings/app-passwords/)
2. Add OAuth consumer
3. **Callback URL**: Auth0's callback
   ```
   https://{your-auth0-domain}/login/callback
   ```
4. Permissions: `account`, `email`
5. Copy **Key** (Client ID) and **Secret**

---

## 5. Environment Variables

### Auth Service (`auth/.env`)

Create `auth/.env` with at least the following settings:
```bash
# Auth0 Configuration
AUTH0_DOMAIN=your-tenant.us.auth0.com
AUTH0_AUDIENCE=https://api.conhub.dev

# Database
DATABASE_URL_NEON=postgresql://neondb_owner:npg_w8jLMEkgsxc9@ep-wispy-credit-aazkw4fu-pooler.westus3.azure.neon.tech/neondb?sslmode=require&channel_binding=require

# JWT Keys (generate with OpenSSL)
JWT_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----
...your private key...
-----END PRIVATE KEY-----"

JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----
...your public key...
-----END PUBLIC KEY-----"

# Redis
REDIS_URL=redis://localhost:6379

# Service Port
AUTH_SERVICE_PORT=3010
```

#### Generate JWT Keys:
```bash
# Generate private key
openssl genrsa -out private.pem 2048

# Extract public key
openssl rsa -in private.pem -pubout -out public.pem

# View keys (copy including BEGIN/END markers)
cat private.pem
cat public.pem
```

### Frontend (`frontend/.env.local`)

Create `frontend/.env.local` with at least the following settings:
```bash
# Auth0 SPA Configuration
NEXT_PUBLIC_AUTH0_DOMAIN=your-tenant.us.auth0.com
NEXT_PUBLIC_AUTH0_CLIENT_ID=your-spa-client-id
NEXT_PUBLIC_AUTH0_AUDIENCE=https://api.conhub.dev
NEXT_PUBLIC_AUTH0_REDIRECT_URI=http://localhost:3000/auth/callback
NEXT_PUBLIC_AUTH0_LOGOUT_REDIRECT_URI=http://localhost:3000/

# ConHub Services
NEXT_PUBLIC_AUTH_SERVICE_URL=http://localhost:3010
NEXT_PUBLIC_BACKEND_BASE_URL=http://localhost:3001

# Feature Flags
NEXT_PUBLIC_AUTH_ENABLED=true
```

---

## 6. Database Migration

Run the Auth0 identity mapping migration:

```bash
# From the database/ directory
sqlx migrate run --database-url "postgresql://neondb_owner:npg_w8jLMEkgsxc9@ep-wispy-credit-aazkw4fu-pooler.westus3.azure.neon.tech/neondb?sslmode=require&channel_binding=require"
```

This adds the `auth0_sub` column to the `users` table for mapping Auth0 identities.

---

## 7. Testing

### Test Auth Service

1. Start the auth service:
```bash
cd auth
cargo run
```

2. Test health endpoint:
```bash
curl http://localhost:3010/health
```

3. Test with a mock Auth0 token (you'll need a real one from Auth0):
```bash
curl -X POST http://localhost:3010/api/auth/auth0/exchange \
  -H "Authorization: Bearer YOUR_AUTH0_ACCESS_TOKEN"
```

### Test Frontend

1. Install dependencies:
```bash
cd frontend
npm install
```

2. Start the frontend:
```bash
npm run dev
```

3. Open browser to `http://localhost:3000`
4. Click "Sign In" - should redirect to Auth0
5. Complete login with email/password or social
6. Should redirect back to `/auth/callback` then to `/dashboard`

### End-to-End Flow

1. User clicks "Sign In" on frontend
2. Frontend redirects to Auth0 with PKCE
3. User authenticates with Auth0 (email/password or social)
4. Auth0 redirects back to `/auth/callback` with authorization code
5. Frontend exchanges code for Auth0 access token
6. Frontend calls `/api/auth/auth0/exchange` with Auth0 token
7. Auth service verifies Auth0 token via JWKS
8. Auth service creates/finds ConHub user
9. Auth service issues ConHub JWT + refresh token
10. Frontend stores tokens and redirects to dashboard

---

## Troubleshooting

### "Auth0 configuration missing"
- Check that all `NEXT_PUBLIC_AUTH0_*` env vars are set in frontend
- Check that `AUTH0_DOMAIN` and `AUTH0_AUDIENCE` are set in auth service

### "JWKS fetch failed"
- Verify `AUTH0_DOMAIN` is correct (no `https://` prefix)
- Check network connectivity to Auth0

### "Invalid state parameter"
- Clear browser localStorage
- Ensure callback URL in Auth0 matches exactly

### "Token validation failed"
- Verify `AUTH0_AUDIENCE` matches between frontend and auth service
- Check that API is authorized in Auth0 SPA settings

### "Email not provided"
- Ensure social connection requests `email` scope
- Check Auth0 user profile has email

---

## Security Checklist

- ✅ Use HTTPS in production
- ✅ Keep `JWT_PRIVATE_KEY` secret
- ✅ Rotate keys periodically
- ✅ Use strong Redis password
- ✅ Enable rate limiting
- ✅ Configure CORS properly
- ✅ Use secure session cookies in production
- ✅ Enable Auth0 brute force protection
- ✅ Enable Auth0 breached password detection
- ✅ Configure Auth0 MFA for admin accounts

---

## Next Steps

1. **Configure Rules/Actions in Auth0** for custom claims
2. **Set up Auth0 Management API** for programmatic user management
3. **Enable MFA** for enhanced security
4. **Configure email templates** in Auth0
5. **Set up monitoring** and alerts
6. **Test token refresh flow**
7. **Implement logout across all sessions**

---

## Support

- Auth0 Documentation: https://auth0.com/docs
- ConHub Issues: [GitHub Issues](https://github.com/your-org/conhub/issues)
