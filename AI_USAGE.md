# AI Usage Disclosure

## AI Tool Used

OpenAI ChatGPT.

## AI-Assisted Development

ChatGPT was used throughout the assessment to assist with:

- Requirement analysis and implementation planning.
- Backend architecture and PostgreSQL schema design.
- Rust and Axum API implementation.
- Password hashing, JWT authentication and email-based 2FA.
- Role-based access control.
- Task creation, assignment and retrieval.
- Per-user caching and cache invalidation.
- React and TypeScript frontend implementation.
- CSS styling and responsive layouts.
- Automated integration test generation.
- API validation commands and project documentation.

Substantial portions of the source code were generated with
AI assistance.

## Manual Implementation and Verification

The following activities were performed manually:

- Local development environment configuration.
- PostgreSQL database and user configuration.
- Dependency installation and project setup.
- Applying implementation code to the local repository.
- Environment variable configuration.
- Running the backend application.
- Executing API requests using curl.
- Verifying user seeding and authentication.
- Creating five tasks through the API.
- Assigning three tasks to James Bond.
- Verifying staff authorization restrictions.
- Confirming cache hits, misses and invalidation.
- Capturing the actual final API response for the README.

## Verified Backend Results

- Admin and James Bond accounts initialized successfully.
- Authentication and two-factor verification completed.
- Five tasks created.
- Three tasks assigned to James Bond.
- Staff task creation rejected with HTTP 403.
- Staff task retrieval returned exactly three assigned tasks.
- Repeated retrieval returned cache.hit as true.
- Task reassignment invalidated the affected cache.

## Implementation Decisions

- PostgreSQL was selected for persistent storage.
- In-memory caching was selected instead of Redis to simplify
  local setup and reduce infrastructure requirements.
- Email delivery is simulated using development email logs.
- Argon2 is used for password and verification code hashing.
- JWT is used for authenticated API requests.
- React hooks, native Fetch API and CSS are used without
  additional frontend frameworks.

## Limitations

AI-generated code may require further review and testing.

The application is designed for local assessment evaluation
and is not intended to be deployed directly into production
without additional security hardening.

## Automated Test Verification

All seven backend integration tests were executed manually
using cargo test -- --test-threads=2.

Results: 7 passed, 0 failed.
Execution time: 19.27 seconds.
