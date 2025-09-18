# Auth Service

A Rust-based authentication service with a companion app service, built using Axum, PostgreSQL, and Redis.

## Project Structure

- `auth-service/` - Main authentication service (port 3000)
- `app-service/` - Companion application service (port 8000)
- `nginx-conf/` - Nginx configuration
- `mount_dir/` - Docker volume mount points

## Prerequisites

- Rust (latest stable)
- Docker & Docker Compose
- PostgreSQL (for local development)
- Redis (for local development)

## Quick Start

### 1. Install Dependencies

```bash
cargo install cargo-watch
```

### 2. Build the Project

```bash
# Build both services
cargo build
```

### 3. Setup Database Migrations

Follow the [SQLx CLI documentation](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query) for offline mode setup.

### 4. Create Required Directories

```bash
mkdir -p mount_dir/postgres mount_dir/dhparam mount_dir/auth-service/html
```

## Running the Services

### Option 1: Docker Compose (Recommended)

```bash
# Start all services
docker compose up

# Visit the services
# App Service: http://localhost:8000
# Auth Service: http://localhost:3000
```

### Option 2: Manual Docker Setup

#### Start Database Services

```bash
# PostgreSQL
docker run --name db \
  -e POSTGRES_PASSWORD={YOUR_SECURE_PASSWORD} \
  --volume ./mount_dir/postgres:/var/lib/postgresql/data \
  -p 5432:5432 \
  -d postgres:15.2-alpine

# Redis
docker run --name redis-db \
  -p "6379:6379" \
  -d redis:7.0-alpine
```

#### Setup Database

```bash
cd auth-service
sqlx database create
cargo sqlx prepare
export SQLX_OFFLINE=true
```

#### Run Services

**App Service:**
```bash
cd app-service
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run
# Visit: http://localhost:8000
```

**Auth Service:**
```bash
cd auth-service
cargo watch -q -c -w src/ -w assets/ -x run
# Visit: http://localhost:3000
```

### Option 3: Using Docker Script

```bash
./docker.sh
# Visit: http://localhost:8000 and http://localhost:3000
```

## Development

### Running Tests

Make sure the database is running for integration tests:

```bash
# Start services
docker compose up -d

# Run tests
cargo test
```

### Hot Reloading

Both services support hot reloading with `cargo watch`:

```bash
# App service with hot reload
cd app-service
cargo watch -q -c -w src/ -w assets/ -w templates/ -x run

# Auth service with hot reload
cd auth-service
cargo watch -q -c -w src/ -w assets/ -x run
```

## Environment Variables

Create a `.env` file with the following variables:

```env
JWT_SECRET=your_jwt_secret_here
APP_SERVICE_HOST=0.0.0.0:8000
POSTGRES_PASSWORD=your_secure_password
DATABASE_URL=postgres://postgres:${POSTGRES_PASSWORD}@localhost:5432
POSTMARK_AUTH_TOKEN=your_postmark_token
EMAIL_SERVICE_HOST=https://api.postmarkapp.com/email
EMAIL_FROM_USER=your_email@domain.com
EMAIL_TIMEOUT_MILLIS=10
SQLX_OFFLINE=true
RUST_LOG=DEBUG
```

## Services

### Auth Service (Port 3000)
- User authentication and authorization
- JWT token management
- Two-factor authentication
- Email verification
- Password hashing and validation

### App Service (Port 8000)
- Main application interface
- User dashboard
- Integration with auth service

## Service Interaction Architecture

The auth-service and app-service follow a **microservices architecture** with clear separation of concerns:

### Architecture Overview

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Browser  │    │   Nginx Proxy   │    │   Auth Service  │
│                 │◄──►│                 │◄──►│   (Port 3000)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   App Service   │    │   PostgreSQL    │
                       │   (Port 8000)   │    │   (Port 5432)   │
                       └─────────────────┘    └─────────────────┘
                                │                        │
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │     Redis       │    │   Email Service │
                       │   (Port 6379)   │    │   (Postmark)    │
                       └─────────────────┘    └─────────────────┘
```

### Communication Flow

#### 1. **App-Service → Auth-Service (Token Validation)**

When a user accesses a protected route in the app-service:

```rust
// app-service/src/main.rs
async fn protected(jar: CookieJar) -> impl IntoResponse {
    // Extract JWT cookie from request
    let jwt_cookie = jar.get("jwt")?;
    
    // Make HTTP POST to auth-service for token validation
    let verify_token_body = serde_json::json!({
        "token": jwt_cookie.value(),
    });
    
    let url = format!("http://{}:3000/verify-token", auth_hostname);
    let response = api_client.post(&url).json(&verify_token_body).send().await?;
    
    // Handle validation response
    match response.status() {
        StatusCode::OK => serve_protected_content(),
        _ => return_unauthorized(),
    }
}
```

#### 2. **Auth-Service CORS Configuration**

The auth-service is configured to accept requests from the app-service:

```rust
// auth-service/src/lib.rs
let allowed_origins = [
    "http://localhost:8000".parse()?,
    format!("http://{}", *APP_SERVICE_HOST).parse()?,
];

let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST])
    .allow_credentials(true)  // Allows cookies
    .allow_origin(allowed_origins);
```

### Authentication Flow

#### Complete User Journey:

1. **User visits app-service** (`app-service.${DOMAIN_NAME}`)
2. **App-service redirects** to auth-service for login
3. **User authenticates** via auth-service (`auth-service.${DOMAIN_NAME}`)
4. **Auth-service sets JWT cookie** and redirects back to app-service
5. **App-service validates token** by calling auth-service's `/verify-token` endpoint
6. **Protected content** is served if token is valid

#### Token Validation Process:

```mermaid
sequenceDiagram
    participant User
    participant AppService
    participant AuthService
    participant Database

    User->>AppService: Request protected route
    AppService->>AppService: Extract JWT cookie
    AppService->>AuthService: POST /verify-token
    AuthService->>Database: Check if token is banned
    AuthService->>AuthService: Validate JWT signature
    AuthService->>AppService: Return validation result
    AppService->>User: Serve protected content or 401
```

### API Endpoints

#### Auth-Service Endpoints:
- `POST /signup` - User registration
- `POST /login` - User authentication
- `POST /logout` - User logout (bans token)
- `POST /verify-2fa` - Two-factor authentication
- `POST /verify-token` - Token validation (used by app-service)

#### App-Service Endpoints:
- `GET /` - Main application interface
- `GET /protected` - Protected route (validates token with auth-service)

### Docker Configuration

#### Service Dependencies:
```yaml
app-service:
  depends_on:
    auth-service:
      condition: service_started
```

#### Network Communication:
```yaml
networks:
  - app-network
```

Both services run on the same Docker network, enabling communication via service hostnames.

### Nginx Reverse Proxy

Nginx routes traffic based on subdomains:

- `app-service.${DOMAIN_NAME}` → `http://app-service:8000`
- `auth-service.${DOMAIN_NAME}` → `http://auth-service:3000`

### Security Features

- **JWT tokens** for stateless authentication
- **CORS** properly configured to prevent unauthorized access
- **SSL/TLS** encryption via Nginx
- **Service isolation** via Docker networking
- **Token validation** on every protected request
- **Banned token storage** in Redis for logout functionality

### Environment Variables

#### App-Service:
```env
AUTH_SERVICE_IP=localhost  # Auth service hostname for internal communication
```

#### Auth-Service:
```env
APP_SERVICE_HOST=0.0.0.0:8000  # App service host for CORS configuration
JWT_SECRET=your_secret_key     # JWT signing secret
```

## Detailed Login Sequence

The following section explains the complete login flow and interaction between the app-service and auth-service.

### 1. Initial User Access

#### User visits App-Service:
```
User → app-service.${DOMAIN_NAME} → App-Service (Port 8000)
```

#### App-Service Response:
The app-service serves the main page with login/logout links that point to the auth-service:

```rust
// app-service/src/main.rs - root() function
async fn root() -> impl IntoResponse {
    let mut address = env::var("AUTH_SERVICE_IP").unwrap_or("localhost".to_owned());
    let login_link = format!("http://{}:3000", address);
    let logout_link = format!("http://{}:3000/logout", address);
    
    // Returns HTML with links to auth-service
    Html(template.render().unwrap())
}
```

### 2. User Redirects to Auth-Service

#### User clicks login link:
```
User → auth-service.${DOMAIN_NAME} → Auth-Service (Port 3000)
```

#### Auth-Service serves login form:
The auth-service presents a login form with email and password fields, plus support for 2FA.

### 3. Login Request Processing

#### Frontend JavaScript:
```javascript
// User submits login form
fetch('/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
})
```

#### Auth-Service Login Handler:
```rust
// auth-service/src/routes/login.rs
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<(StatusCode, Json<LoginResponse>), AuthAPIError>)
```

**Login Process:**
1. **Parse credentials** (email/password validation)
2. **Validate user** against PostgreSQL database
3. **Check 2FA requirement** from user record
4. **Branch based on 2FA status**

### 4A. Login Without 2FA (Simple Flow)

#### Auth-Service Response:
```rust
// auth-service/src/routes/login.rs - handle_no_2fa()
async fn handle_no_2fa(email: &Email, jar: CookieJar) -> (CookieJar, Result<...>) {
    let auth_cookie = generate_auth_cookie(email)?;  // Creates JWT token
    let updated_jar = jar.add(auth_cookie);          // Sets JWT cookie
    
    return (updated_jar, Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))));
}
```

#### Frontend Response Handling:
```javascript
// app.js - login response handling
if (response.status === 200) {
    alert("You have successfully logged in.");
    // User is now authenticated
}
```

### 4B. Login With 2FA (Two-Step Flow)

#### Step 1: Initial Login Response:
```rust
// auth-service/src/routes/login.rs - handle_2fa()
async fn handle_2fa(email: &Email, state: &AppState, jar: CookieJar) -> (CookieJar, Result<...>) {
    let login_attempt_id = LoginAttemptId::default();
    let tw_code = TwoFACode::default();
    
    // Store 2FA code in Redis
    state.two_fa_code_store.write().await
        .add_code(email.clone(), login_attempt_id.clone(), tw_code.clone()).await?;
    
    // Send 2FA code via email
    state.email_client.write().await
        .send_email(email, "2FA Code", tw_code.as_ref()).await?;
    
    return (jar, Ok((StatusCode::PARTIAL_CONTENT, Json(LoginResponse::TwoFactorAuth(response)))));
}
```

#### Frontend 2FA Form Display:
```javascript
// app.js - 2FA response handling
if (response.status === 206) {
    TwoFAForm.email.value = email;
    TwoFAForm.login_attempt_id.value = data.loginAttemptId;
    
    // Show 2FA form, hide login form
    loginSection.style.display = "none";
    twoFASection.style.display = "block";
}
```

#### Step 2: 2FA Verification:
```javascript
// User submits 2FA code
fetch('/verify-2fa', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ 
        email, 
        loginAttemptId, 
        "2FACode": TwoFACode 
    }),
})
```

#### Auth-Service 2FA Verification:
```rust
// auth-service/src/routes/verify_2fa.rs
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<StatusCode, AuthAPIError>) {
    // Validate 2FA code against Redis store
    let two_fa_code_store = state.two_fa_code_store.read().await;
    match two_fa_code_store.get_code(&email).await {
        Ok((id, code)) if id == login_attempt_id && code == two_fa_code => {
            // Code is valid - generate JWT cookie
            let auth_cookie = generate_auth_cookie(&email)?;
            let updated_jar = jar.add(auth_cookie);
            
            // Remove used 2FA code
            state.two_fa_code_store.write().await.remove_code(&email).await?;
            
            return (updated_jar, Ok(StatusCode::OK));
        }
        _ => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    }
}
```

### 5. Accessing Protected Routes

#### User visits protected route in App-Service:
```
User → app-service.${DOMAIN_NAME}/protected → App-Service
```

#### App-Service Token Validation:
```rust
// app-service/src/main.rs - protected()
async fn protected(jar: CookieJar) -> impl IntoResponse {
    let jwt_cookie = jar.get("jwt")?;  // Extract JWT cookie
    
    // Call auth-service to validate token
    let verify_token_body = serde_json::json!({
        "token": jwt_cookie.value(),
    });
    
    let url = format!("http://{}:3000/verify-token", auth_hostname);
    let response = api_client.post(&url).json(&verify_token_body).send().await?;
    
    match response.status() {
        StatusCode::OK => serve_protected_content(),
        _ => return_unauthorized(),
    }
}
```

#### Auth-Service Token Verification:
```rust
// auth-service/src/routes/verify_token.rs
pub async fn verify_token(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    // Check if token is banned in Redis
    {
        let banned_token_store = state.banned_token_store.read().await;
        match banned_token_store.get_token(&req_token).await {
            Ok(_) => return Err(AuthAPIError::InvalidToken),  // Token is banned
            Err(_) => Ok(()),  // Token not banned
        }?;
    }
    
    // Validate JWT signature and expiration
    match validate_token(&token, state.banned_token_store.clone()).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
```

### 6. Complete Login Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant AppService
    participant AuthService
    participant Database
    participant Redis
    participant Email

    User->>AppService: Visit app-service.${DOMAIN_NAME}
    AppService->>User: Return page with login link
    
    User->>AuthService: Click login link → auth-service.${DOMAIN_NAME}
    AuthService->>User: Return login form
    
    User->>AuthService: POST /login {email, password}
    AuthService->>Database: Validate user credentials
    Database->>AuthService: Return user data
    
    alt User has 2FA enabled
        AuthService->>Redis: Store 2FA code + login_attempt_id
        AuthService->>Email: Send 2FA code
        AuthService->>User: Return 206 + login_attempt_id
        User->>AuthService: POST /verify-2fa {email, login_attempt_id, 2FACode}
        AuthService->>Redis: Validate 2FA code
        Redis->>AuthService: Return validation result
        AuthService->>Redis: Remove used 2FA code
    end
    
    AuthService->>AuthService: Generate JWT token
    AuthService->>User: Set JWT cookie + return 200
    
    User->>AppService: Visit /protected route
    AppService->>AppService: Extract JWT cookie
    AppService->>AuthService: POST /verify-token {token}
    AuthService->>Redis: Check if token is banned
    AuthService->>AuthService: Validate JWT signature
    AuthService->>AppService: Return validation result
    AppService->>User: Serve protected content or 401
```

### 7. Key Interaction Points

1. **Initial Redirect**: App-service provides login links to auth-service
2. **Authentication**: Auth-service handles all authentication logic
3. **Token Generation**: Auth-service creates JWT tokens after successful auth
4. **Cookie Management**: JWT cookies are set by auth-service, read by app-service
5. **Token Validation**: App-service validates tokens by calling auth-service
6. **2FA Flow**: Multi-step authentication with email verification
7. **Security**: Banned tokens stored in Redis, validated on each request

### 8. Security Features

- **JWT Tokens**: Stateless authentication with expiration
- **2FA Support**: Email-based two-factor authentication
- **Token Banning**: Logout functionality bans tokens in Redis
- **CORS Protection**: Auth-service only accepts requests from app-service
- **Input Validation**: All inputs are validated and sanitized
- **Secure Cookies**: JWT cookies are HTTP-only and secure

This detailed login sequence shows how the microservices architecture enables clean separation of concerns while maintaining secure authentication flows.

## Docker Volumes

The following directories are used for Docker volume mounts:

- `mount_dir/postgres/` - PostgreSQL data persistence
- `mount_dir/dhparam/` - SSL DH parameters
- `mount_dir/auth-service/html/` - Web server files

## Troubleshooting

### Common Issues

1. **Docker Compose fails with mount errors**: Ensure all required directories exist (see Quick Start section)

2. **Database connection issues**: Verify PostgreSQL is running and credentials are correct

3. **SSL certificate errors**: This is expected on first run - certificates will be generated by certbot

### Logs

View service logs:

```bash
# All services
docker compose logs

# Specific service
docker compose logs auth-service
docker compose logs app-service
```