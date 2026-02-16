[![codecov](https://codecov.io/gh/jordimarsal/rustcut/branch/main/graph/badge.svg)](https://codecov.io/gh/jordimarsal/rustcut)

# rustcut
A small, fast URL shortener written in **Rust** — Actix-Web + SQLx (SQLite), built with DDD / Hexagonal principles.

---

## ✨ Overview
`rustcut` provides a minimal HTTP API to create short URLs, perform redirects, and manage users and URLs. The codebase uses ports/adapters (domain traits + SQLx adapters) so business logic is decoupled from persistence.

## Key features
- Create / list / delete users (each user receives an API key)
- Create short URLs and redirect users (303 See Other)
- Admin endpoints to inspect or delete URLs using a secret key
- Persistence with **SQLite** (sqlx); in-memory DB used for integration tests
- Unit + integration tests and Codecov integration

## Tech stack
- Rust (edition 2021)
- actix-web, tokio
- sqlx (SQLite), async-trait
- serde / serde_json
- log + log4rs (env_logger as fallback)

## Quickstart (development)
Prerequisites: Rust toolchain (stable).

1. Clone the repo:

   ```sh
   git clone https://github.com/jordimarsal/rustcut.git
   cd rustcut
   ```

2. Build and run (defaults shown):

   ```sh
   cargo build
   BASE_URL=127.0.0.1 SERVER_PORT=8083 PROTOCOL=http cargo run
   ```

3. Example requests:

   - Create a user:
     ```sh
     curl -X POST -H "Content-Type: application/json" \
       -d '{"username":"alice","email":"a@x.com"}' http://127.0.0.1:8083/users
     ```

   - Create a short URL (use `api_key` returned when creating a user):
     ```sh
     curl -X POST -H "Content-Type: application/json" \
       -d '{"api_key":"<API_KEY>", "target_url":"https://example.com"}' \
       http://127.0.0.1:8083/url
     ```

   - Redirect (browser or `curl -v`):
     ```sh
     curl -v http://127.0.0.1:8083/<SHORT_KEY>
     ```

## Configuration
Environment variables (defaults):
- `BASE_URL` — default `localhost`
- `SERVER_PORT` — default `8080`
- `PROTOCOL` — default `https`

The app reads `.env` in normal runs (not during `cargo test`). The logger is configured by `log4rs.yml` with an env_logger fallback.

## HTTP API (summary)
- POST `/users` — create user
  - body: `{ "username": "..", "email": ".." }`
  - returns: `{ user: {...}, api_key: "..." }`

- GET `/users` — list users

- DELETE `/users/{id}` — delete user

- POST `/url` — create short URL
  - body: `{ "api_key": "..", "target_url": "https://..." }`
  - returns: `URLInfoDto { target_url, is_active, clicks, url, admin_url }`

- GET `/{url_key}` — redirect (303)

- GET `/admin/{secret_key}` — get admin URL info

- DELETE `/admin/{secret_key}` — delete URL and return admin DTO

(See controller tests in `src/*/application/controllers/*` for examples.)

## Tests & coverage
- Run tests:
  ```sh
  cargo test -- --nocapture
  ```

- Generate LCOV (local / CI):
  ```sh
  cargo llvm-cov --lcov --output-path coverage/lcov.info
  ```

- CI uploads `coverage/lcov.info` to Codecov (see `.github/workflows/coverage.yml`). Add `CODECOV_TOKEN` to GitHub Secrets only for private repos.

## Architecture
- Domain defines `ports` (traits) and pure domain models.
- `infra` contains SQLx adapters that implement repository ports.
- Application layer implements controllers, DTOs and mappers.
- Design priorities: testability, separation of concerns, and the ability to swap persistence implementations.

## Contributing
- Fork → branch → PR
- Add tests for new behavior
- Run `cargo fmt` and `cargo clippy`

## Troubleshooting
- The CLI parser strips common test-harness flags (e.g. `--nocapture`).
- If Codecov upload fails: make sure the repo is enabled on codecov.io and `CODECOV_TOKEN` is set in GitHub Secrets for private repositories.

## License
MIT — see `LICENSE`

---

Want OpenAPI documentation, a Postman collection, or a `docker-compose` example added? I can add any of those on request."