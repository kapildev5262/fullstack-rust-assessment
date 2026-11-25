# Task Management System

A full-stack task management application with role-based access
control, email-based two-factor authentication, PostgreSQL persistence
and per-user task caching.

## Technology Stack

### Backend

- Rust
- Axum
- Tokio
- PostgreSQL
- SQLx
- JWT authentication
- Argon2 password hashing
- Serde

### Frontend

- React
- TypeScript
- Vite
- CSS
- Fetch API

## Features

- User authentication with email and password.
- Two-step authentication using a six-digit verification code.
- Verification code expiry, one-time usage and attempt limits.
- JWT-based authenticated API requests.
- Admin-only task creation and assignment.
- Staff access restricted to their assigned tasks.
- PostgreSQL-backed persistent storage.
- Per-user in-memory caching with assignment-based invalidation.
- Responsive admin and staff dashboards.
- Development email inbox for simulated email delivery.
- Automated backend integration tests.

## Architecture

React + TypeScript
        |
        | HTTP / JSON / JWT
        v
Rust + Axum API
        |
        | Authentication / Authorization
        |
        +---- In-memory per-user task cache
        |
        v
PostgreSQL via SQLx

The backend is responsible for authentication, authorization,
business logic, database access and cache consistency.

The frontend uses a separate typed API client and role-specific
components.

## Project Structure

    fullstack-rust-assessment/
    ├── backend/
    │   ├── migrations/
    │   │   └── 0001_init.sql
    │   ├── src/
    │   │   ├── routes/
    │   │   │   ├── auth.rs
    │   │   │   ├── users.rs
    │   │   │   ├── tasks.rs
    │   │   │   └── mod.rs
    │   │   ├── auth.rs
    │   │   ├── error.rs
    │   │   ├── models.rs
    │   │   ├── state.rs
    │   │   ├── lib.rs
    │   │   └── main.rs
    │   ├── tests/
    │   │   └── workflow.rs
    │   ├── Cargo.toml
    │   └── .env.example
    ├── frontend/
    │   ├── src/
    │   │   ├── components/
    │   │   │   ├── Login.tsx
    │   │   │   ├── AdminDashboard.tsx
    │   │   │   └── StaffDashboard.tsx
    │   │   ├── api.ts
    │   │   ├── types.ts
    │   │   ├── App.tsx
    │   │   ├── App.css
    │   │   └── main.tsx
    │   ├── package.json
    │   └── .env.example
    ├── screenshots/
    ├── AI_USAGE.md
    └── README.md

## Prerequisites

- Rust stable and Cargo
- Node.js compatible with the installed Vite version
- npm
- PostgreSQL
- Git
- curl and jq (optional, for API verification)

## Local Setup

### 1. Clone the repository

Clone this repository and navigate to its root directory.

### 2. Configure PostgreSQL

Start PostgreSQL and open the PostgreSQL console.

    sudo systemctl start postgresql
    sudo -u postgres psql

Create a local database and database user.

    CREATE USER task_user WITH PASSWORD 'task_password';
    CREATE DATABASE task_manager OWNER task_user;

Exit the PostgreSQL console.

    \q

These credentials are for local development only.

### 3. Configure and start the backend

    cd backend
    cp .env.example .env

Configure the following environment variables in backend/.env:

    DATABASE_URL=postgres://task_user:task_password@localhost:5432/task_manager
    JWT_SECRET=replace_with_a_secure_random_secret
    SERVER_HOST=127.0.0.1
    SERVER_PORT=8080
    FRONTEND_URL=http://localhost:5173
    APP_ENV=development

Generate a JWT secret using:

    openssl rand -hex 32

Replace JWT_SECRET with the generated value.

Start the backend:

    cargo run

Database migrations are automatically applied during startup.

Backend URL: http://localhost:8080

Health check:

    curl http://localhost:8080/health

### 4. Configure and start the frontend

Open a separate terminal from the repository root.

    cd frontend
    cp .env.example .env
    npm ci
    npm run dev -- --host localhost --port 5173 --strictPort

Frontend URL: http://localhost:5173

## Demo Users

Initialize the demo users using:

    curl -X POST http://localhost:8080/seed/users

Alternatively, use the Initialize demo users button on the login page.

| User | Email | Password | Role |
|------|-------|----------|------|
| Admin | admin@example.com | Admin@123 | admin |
| James Bond | jamesbond@example.com | James@123 | staff |

These are development-only credentials.

The seed endpoint is idempotent with respect to user records.

## API Endpoints

| Method | Endpoint | Access |
|--------|----------|--------|
| GET | /health | Public |
| POST | /seed/users | Development only |
| POST | /auth/login | Public |
| POST | /auth/verify-2fa | Public |
| GET | /dev/email-logs/latest | Development only |
| POST | /tasks | Admin |
| GET | /tasks | Admin |
| POST | /tasks/assign | Admin |
| GET | /tasks/view-my-tasks | Authenticated |

Protected endpoints require:

    Authorization: Bearer <access_token>

## Authentication Flow

1. Submit email and password to POST /auth/login.
2. The server verifies the password using Argon2.
3. The server creates a login challenge and generates a six-digit code.
4. The code is delivered through the simulated development email service.
5. Submit the challenge ID and code to POST /auth/verify-2fa.
6. The server validates and consumes the challenge.
7. A JWT access token is returned only after successful verification.

Verification codes expire after five minutes and cannot be reused.
Each challenge permits a maximum of five failed verification attempts.

Passwords and persisted verification codes are stored as hashes.

JWT access tokens expire after one hour.

## Assessment Validation Workflow

Start with an empty task database.

1. Initialize the two demo users.
2. Log in as Admin.
3. Retrieve the verification code using the development inbox.
4. Complete two-factor authentication.
5. Create exactly five tasks.
6. Assign exactly three tasks to James Bond.
7. Log out from the Admin account.
8. Log in as James Bond and complete two-factor authentication.
9. Attempt task creation using the staff dashboard.
10. Confirm that the API returns HTTP 403 Forbidden.
11. Confirm that exactly three assigned tasks are displayed.
12. Refresh the assigned task list and confirm a cache hit.

Example task dataset:

| Task | Priority | Assigned to James Bond |
|------|----------|------------------------|
| Implement REST API | High | Yes |
| Design database schema | Medium | Yes |
| Build login screen | Low | Yes |
| Write documentation | Medium | No |
| Run integration tests | High | No |

The task counts are assessment validation requirements, not
hardcoded application limits.

## Example API Requests

### Login

    curl -X POST http://localhost:8080/auth/login \
      -H "Content-Type: application/json" \
      -d '{"email":"admin@example.com","password":"Admin@123"}'

### Retrieve development verification code

    curl http://localhost:8080/dev/email-logs/latest

### Verify 2FA

    curl -X POST http://localhost:8080/auth/verify-2fa \
      -H "Content-Type: application/json" \
      -d '{"login_challenge_id":"CHALLENGE_UUID","code":"CODE"}'

### Create a task

    curl -X POST http://localhost:8080/tasks \
      -H "Authorization: Bearer ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"title":"Implement REST API","description":"Assessment task","priority":"high"}'

### Assign tasks

    curl -X POST http://localhost:8080/tasks/assign \
      -H "Authorization: Bearer ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"task_ids":["TASK_UUID_1","TASK_UUID_2","TASK_UUID_3"],"assigned_to_email":"jamesbond@example.com"}'

### View assigned tasks

    curl http://localhost:8080/tasks/view-my-tasks \
      -H "Authorization: Bearer STAFF_TOKEN"

Replace the placeholder tokens, UUIDs and verification codes
with actual values returned by the API.

## Caching Strategy

The backend uses an in-memory HashMap protected by a Tokio RwLock.

Cache keys are authenticated user UUIDs.

- First request: retrieve tasks from PostgreSQL and populate the cache.
- Subsequent requests: return cached tasks.
- Assignment or reassignment: invalidate affected users' cache entries.
- Server restart: clear the in-memory cache.

Assignment transactions and cache population are coordinated to
prevent stale cache entries from being inserted during assignment.

PostgreSQL remains the source of truth.

In-memory caching was selected to reduce infrastructure requirements
for this assessment. Redis would be appropriate for a multi-instance
deployment.

## Database Design

### users

Stores user identity, email, hashed password, role and timestamps.

### tasks

Stores task details, priority, status, creator, assignee and timestamps.

### login_challenges

Stores challenge identity, associated user, hashed verification code,
expiry timestamp, consumption timestamp and failed attempt count.

Foreign keys enforce relationships between users and tasks.
Database constraints enforce valid roles, priorities and statuses.

## Automated Tests

SQLx integration tests use isolated PostgreSQL databases.

For local testing, grant the database user permission to create databases:

    sudo -u postgres psql -c "ALTER ROLE task_user CREATEDB;"

From the backend directory:

    set -a
    source .env
    set +a
    cargo test -- --test-threads=2

The integration test suite covers:

- Idempotent user seeding.
- Invalid login credentials.
- Login challenge creation without issuing a JWT.
- Incorrect, correct, expired and reused verification codes.
- Verification attempt limits.
- Missing and invalid JWTs.
- Admin task creation and assignment.
- Staff authorization restrictions.
- Assigned task filtering.
- Cache hits, misses, isolation and invalidation.
- Complete five-task and three-assignment workflow.

The tests exercise the actual Axum router and PostgreSQL queries.

## Code Quality

Backend:

    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

Frontend:

    npm run build
    npm run lint

## Security Considerations and Limitations

- Development email delivery is simulated rather than using SMTP.
- Development seed and email-log endpoints must not be enabled
  in production.
- The frontend includes demo credentials for local evaluation.
- JWTs are stored in sessionStorage, which remains accessible
  to JavaScript.
- Expired sessions require the user to log in again.
- In-memory caching is limited to a single application instance
  and does not implement TTL-based expiration.
- Global login rate limiting and production-grade monitoring
  are outside the current implementation.
- The frontend exposes James Bond as the predefined assignee.
- Task editing and deletion are outside the assessment workflow.

Production deployment would require HTTPS, real email delivery,
stronger session management, distributed caching, rate limiting
and additional security hardening.

## AI Usage

See [AI_USAGE.md](AI_USAGE.md) for details of AI-assisted development.

## Actual Final API Validation Response

Endpoint: `GET /tasks/view-my-tasks`

HTTP status: `200 OK`

```json
{
  "cache": {
    "hit": true
  },
  "summary": {
    "total_assigned_tasks": 3
  },
  "tasks": [
    {
      "assigned_to": "jamesbond@example.com",
      "id": "54ff20f3-085e-4f60-9c03-f3abeeac1bb9",
      "priority": "high",
      "status": "todo",
      "title": "Implement REST API"
    },
    {
      "assigned_to": "jamesbond@example.com",
      "id": "90da702c-9bf1-4fe7-8387-2e9c8d62ae7f",
      "priority": "medium",
      "status": "todo",
      "title": "Design database schema"
    },
    {
      "assigned_to": "jamesbond@example.com",
      "id": "c6029b7a-2ed0-41e1-9339-9e1fd0f35a74",
      "priority": "low",
      "status": "todo",
      "title": "Build login screen"
    }
  ],
  "user": {
    "email": "jamesbond@example.com",
    "role": "staff"
  }
}

```

## Screenshots

### Admin Dashboard

![Admin Dashboard](screenshots/01-admin-dashboard.png)

### Staff Dashboard

![Staff Dashboard](screenshots/02-staff-dashboard.png)

### Backend Implementation and Tests

![Backend Tests](screenshots/03-backend-tests.png)
