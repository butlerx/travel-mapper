# Setup and Configuration Guide

This guide covers everything you need to get Travel Mapper running — from a
minimal development setup to a fully configured production deployment with
flight/rail status enrichment, email verification, push notifications, and
attachment storage.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Core Configuration](#core-configuration)
  - [TripIt API credentials](#tripit-api-credentials)
  - [Encryption key](#encryption-key)
  - [Database](#database)
  - [Server port](#server-port)
- [Architecture Overview](#architecture-overview)
  - [Server binary](#server-binary)
  - [Sync worker binary](#sync-worker-binary)
  - [Seed binary](#seed-binary)
- [Running the Application](#running-the-application)
  - [Development mode](#development-mode)
  - [Production mode](#production-mode)
- [Database Management](#database-management)
  - [Migrations](#migrations)
  - [SQLite configuration](#sqlite-configuration)
  - [Resetting the database](#resetting-the-database)
  - [Query cache](#query-cache)
- [Optional Integrations](#optional-integrations)
  - [Flight status enrichment (AirLabs)](#flight-status-enrichment-airlabs)
  - [Flight route verification (OpenSky)](#flight-route-verification-opensky)
  - [Rail status enrichment](#rail-status-enrichment)
- [Email and SMTP](#email-and-smtp)
- [Web Push Notifications (VAPID)](#web-push-notifications-vapid)
- [Attachment Storage](#attachment-storage)
- [User Registration](#user-registration)
- [Logging and Telemetry](#logging-and-telemetry)
- [Security Considerations](#security-considerations)
- [Task Runner Reference](#task-runner-reference)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

- **[mise](https://mise.jdx.dev/)** — manages tool versions (Rust, Node, sqlx,
  cargo-watch, etc.) and runs project tasks. Install it first:

  ```bash
  curl https://mise.run | sh
  ```

- **A [TripIt developer account](https://www.tripit.com/developer)** — required
  for the TripIt sync integration. If you only plan to use CSV import, you still
  need to provide consumer key/secret values (they can be dummy values), because
  the server binary requires them at startup.

mise will automatically install all other tools when you run `mise install`:

| Tool             | Purpose                             |
| ---------------- | ----------------------------------- |
| Rust (stable)    | Compiler + rust-analyzer            |
| Node (LTS)       | Prettier, TypeScript type-checking  |
| sqlx-cli         | Database creation, migrations, query cache |
| cargo-watch      | Auto-reload server on file changes  |
| cargo-nextest    | Test runner                         |
| cargo-binstall   | Fast binary installation            |
| taplo            | TOML formatting                     |
| prettier         | JS/CSS formatting                   |
| typescript       | JS type-checking                    |

## Installation

```bash
git clone https://github.com/butlerx/travel-export
cd travel-export
mise install          # installs all tools defined in .mise.toml
mise run build        # debug build
```

For an optimised release binary:

```bash
mise run build:release
```

## Core Configuration

All configuration is done through environment variables. Copy the example file
and fill in your values:

```bash
cp .env.example .env
```

mise automatically loads `.env` in the project directory (configured via
`[env] _.file = ".env"` in `.mise.toml`). Every environment variable can also be
passed as a CLI flag — run `./target/debug/server --help` for the full list.

### TripIt API credentials

**Required.** Both the server and sync worker need these to authenticate with
the TripIt API.

1. Go to [tripit.com/developer](https://www.tripit.com/developer)
2. Create a new app (name and description can be anything)
3. Copy the **Consumer Key** and **Consumer Secret**

```bash
TRIPIT_CONSUMER_KEY=your_consumer_key
TRIPIT_CONSUMER_SECRET=your_consumer_secret
```

### Encryption key

**Required.** A 32-byte key (64 hex characters) used for AES-256-GCM encryption
of TripIt OAuth tokens stored in the database.

Generate one:

```bash
openssl rand -hex 32
```

```bash
ENCRYPTION_KEY=a1b2c3d4e5f6...   # 64 hex chars
```

The server will refuse to start if the key is missing or not exactly 64 hex
characters. **Keep this key safe** — if you lose it, all encrypted credentials
in the database become unrecoverable.

### Database

**Optional.** Defaults to `sqlite:travel.db` in the working directory.

```bash
DATABASE_URL=sqlite:travel.db
```

The database file is created automatically on first run. Migrations run
automatically when the connection pool is initialised — no manual migration step
is needed for normal operation.

### Server port

**Optional.** Defaults to `3000`.

```bash
PORT=3000
```

The server binds to `0.0.0.0:{PORT}`, accepting connections on all interfaces.

---

## Architecture Overview

Travel Mapper consists of three binaries that share the same database and
configuration:

### Server binary

```bash
cargo run --bin server
# or: mise run serve   (with cargo-watch auto-reload)
```

The main Axum HTTP server. Serves the web UI (Leptos SSR), REST API, Swagger UI
(`/docs`), calendar feeds, and static assets. Handles user authentication,
journey CRUD, trip grouping, CSV import, and TripIt OAuth flows.

The server can also process sync jobs inline (for API requests) but normally
defers to the worker.

### Sync worker binary

```bash
cargo run --bin sync-worker
# or: mise run worker
```

A background process that polls the `sync_jobs` table at a configurable interval
(default: every 5 seconds). When a pending job is found, it:

1. Decrypts the user's stored TripIt OAuth tokens
2. Fetches all trips and journeys from the TripIt API
3. Geocodes origin/destination addresses
4. Stores/updates journeys in the database
5. Enriches flights with status data (if AirLabs/OpenSky configured)
6. Enriches rail journeys with live status (if rail providers configured)
7. Sends Web Push notifications on completion (if VAPID configured)

```bash
SYNC_POLL_INTERVAL_SECS=5    # how often the worker checks for jobs (default: 5)
```

### Seed binary

```bash
cargo run --bin seed
# or: mise run seed
```

Creates a test user (`test` / `test`) with sample journeys and trips for local
development. Idempotent — safe to run multiple times (skips user creation if
the user already exists).

The seed script also accepts optional TripIt access tokens to pre-populate
credentials for the test user:

```bash
TRIPIT_ACCESS_TOKEN=your_access_token
TRIPIT_ACCESS_TOKEN_SECRET=your_access_token_secret
```

These are only used by the seed script and are not needed for normal operation.

---

## Running the Application

### Development mode

The fastest way to get everything running:

```bash
mise run dev
```

This starts three processes concurrently:

1. **seed** — creates the test user and sample data (runs migrations first)
2. **serve** — starts the server with `cargo watch` for auto-reload on code
   changes
3. **worker** — starts the background sync worker

Visit `http://localhost:3000`. Log in with `test` / `test`.

To run components individually:

```bash
mise run serve     # server only (with auto-reload)
mise run worker    # sync worker only
mise run seed      # seed data only
```

### Production mode

Build an optimised release binary:

```bash
mise run build:release
```

This produces three binaries in `target/release/`:

- `server`
- `sync-worker`
- `seed`

Run them directly:

```bash
# Required env vars must be set or passed as flags
export TRIPIT_CONSUMER_KEY=...
export TRIPIT_CONSUMER_SECRET=...
export ENCRYPTION_KEY=...
export DATABASE_URL=sqlite:/var/lib/travel-mapper/travel.db

./target/release/server &
./target/release/sync-worker &
```

Both binaries support graceful shutdown via `Ctrl+C` (SIGINT).

---

## Database Management

### Migrations

Migrations live in `migrations/` as sequentially numbered SQL files
(`001_initial.sql`, `002_add_trips.sql`, etc.). They run **automatically** when
the application starts — the `create_pool()` function applies all pending
migrations before returning the connection pool.

For manual migration management:

```bash
mise run db:create     # create the SQLite file
mise run db:migrate    # run pending migrations
mise run db:reset      # drop, recreate, and migrate from scratch
```

**Never modify a committed migration file.** If you need a schema change, create
a new migration.

### SQLite configuration

The application configures SQLite with:

- **WAL journal mode** — enables concurrent reads during writes
- **busy_timeout = 5000ms** — retries on lock contention instead of failing
  immediately
- **Max 5 connections** — pool limit appropriate for SQLite's concurrency model

These are set automatically via PRAGMAs at pool creation time.

### Resetting the database

```bash
mise run db:reset
```

This drops the database file, recreates it, and runs all migrations from
scratch.

### Query cache

Travel Mapper uses sqlx compile-time checked queries (`sqlx::query!()` and
`sqlx::query_as!()`). The `.sqlx/` directory contains cached query metadata that
allows the project to build without a live database connection.

**After changing any SQL query**, regenerate the cache:

```bash
mise run db:prepare
```

This runs migrations first, then generates fresh `.sqlx/*.json` files. Commit
these files — they're needed for CI builds and fresh checkouts.

---

## Optional Integrations

All optional integrations degrade gracefully — if their API keys are not set,
the corresponding features are simply skipped.

### Flight status enrichment (AirLabs)

Provides live and historical flight status data (delays, cancellations, gate
changes) for air journeys.

```bash
AIRLABS_API_KEY=your_airlabs_key
```

Get an API key from [airlabs.co](https://airlabs.co/).

### Flight route verification (OpenSky)

Verifies flight routes against ADS-B data from the
[OpenSky Network](https://opensky-network.org/).

```bash
OPENSKY_CLIENT_ID=your_client_id
OPENSKY_CLIENT_SECRET=your_client_secret
```

Register for API access at
[opensky-network.org](https://opensky-network.org/).

### Rail status enrichment

Rail status providers are selected automatically based on the journey's
origin/destination country:

| Country | Provider   | Required variables                   |
| ------- | ---------- | ------------------------------------ |
| GB      | Darwin     | `DARWIN_API_TOKEN`                   |
| DE      | DB RIS     | `DB_RIS_API_KEY`, `DB_RIS_CLIENT_ID` |
| US      | Amtrak     | _(not yet implemented)_              |
| Other   | Transitland | `TRANSITLAND_API_KEY`               |

Cross-country rail journeys (different origin and destination countries) always
use Transitland.

If the required API key for a particular provider is not configured, enrichment
for that country's rail journeys is silently skipped.

#### National Rail Darwin (UK)

Live rail status for journeys within Great Britain.

```bash
DARWIN_API_TOKEN=your_darwin_token
```

Register at the
[National Rail Data Portal](https://www.nationalrail.co.uk/developers/).

#### DB RIS (Germany)

Live rail status for journeys within Germany (Deutsche Bahn). Requires both an
API key and a client ID.

```bash
DB_RIS_API_KEY=your_db_ris_key
DB_RIS_CLIENT_ID=your_db_ris_client_id
```

Register at the
[DB API Marketplace](https://developers.deutschebahn.com/).

#### Transitland (Global)

GTFS-RT based rail status for any country not handled by a country-specific
provider.

```bash
TRANSITLAND_API_KEY=your_transitland_key
```

Get an API key from [transit.land](https://www.transit.land/).

---

## Email and SMTP

Email is used for account verification. **All four SMTP fields must be set** to
enable email — if any are missing, email sending is silently disabled.

```bash
SMTP_HOST=smtp.example.com
SMTP_PORT=587                    # default: 587 (STARTTLS)
SMTP_USERNAME=your_username
SMTP_PASSWORD=your_password
EMAIL_FROM=noreply@example.com
```

When SMTP is not configured:

- Email verification is not available
- **Registration is automatically disabled** (even if `REGISTRATION_ENABLED=true`),
  because the system cannot verify email addresses. A warning is logged:
  `REGISTRATION_ENABLED is true but SMTP is not configured — registration disabled`

The SMTP connection uses STARTTLS on the configured port. The `lettre` crate
handles transport with native TLS.

---

## Web Push Notifications (VAPID)

Web Push notifications alert users when sync jobs complete. Both VAPID fields
must be set to enable this feature.

```bash
VAPID_PRIVATE_KEY_PATH=/path/to/vapid-private.pem
VAPID_PUBLIC_KEY=BFkX3...base64url...
```

### Generating VAPID keys

```bash
# Generate a private key
openssl ecparam -name prime256v1 -genkey -noout -out vapid-private.pem

# Extract the public key in the format browsers expect (base64url, uncompressed)
openssl ec -in vapid-private.pem -pubout -outform DER 2>/dev/null \
  | tail -c 65 \
  | base64 | tr '+/' '-_' | tr -d '='
```

Set `VAPID_PRIVATE_KEY_PATH` to the path of the PEM file, and
`VAPID_PUBLIC_KEY` to the base64url-encoded public key string.

When VAPID is not configured, the push notification UI is hidden from users and
the worker skips push delivery.

---

## Attachment Storage

To enable photo/document attachments on journeys, set a filesystem directory:

```bash
ATTACHMENTS_PATH=/var/lib/travel-mapper/attachments
```

The directory must exist and be writable by the server process. When unset,
attachment upload endpoints return an error and the UI hides upload controls.

---

## User Registration

```bash
REGISTRATION_ENABLED=true     # default: true
```

Controls whether new users can register via the `/register` page or
`POST /auth/register` API.

**Important behavioural note:** Even when set to `true`, registration is
**automatically disabled** if SMTP is not configured. This is because the system
requires email verification for new accounts. A warning is logged when this
happens.

To run a single-user instance, register your user, then set:

```bash
REGISTRATION_ENABLED=false
```

---

## Logging and Telemetry

Travel Mapper uses the `tracing` crate with structured logging.

```bash
RUST_LOG=info,tower_http=debug    # default
```

The format differs by build type:

| Build   | Format             | Example                                      |
| ------- | ------------------ | -------------------------------------------- |
| Debug   | Pretty (coloured)  | Human-readable, multi-line with span events  |
| Release | JSON               | Machine-parseable, one JSON object per line  |

The filter is read from the `RUST_LOG` environment variable and follows the
standard
[`tracing-subscriber` EnvFilter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html):

```bash
# Show debug logs for the worker module only
RUST_LOG=info,travel_mapper::worker=debug

# Silence tower_http request logging
RUST_LOG=info,tower_http=warn

# Verbose everything
RUST_LOG=trace
```

---

## Security Considerations

### Encryption key

The `ENCRYPTION_KEY` protects TripIt OAuth tokens at rest using AES-256-GCM.
Treat it like a database encryption key:

- Generate a cryptographically random key (`openssl rand -hex 32`)
- Never commit it to version control
- Back it up securely — losing it means re-authenticating all users with TripIt

### Password hashing

Passwords are hashed using Argon2 (the `argon2` crate with default parameters).
No plaintext passwords are stored.

### Session management

Sessions are cookie-based (`session_id` cookie). API keys are an alternative
authentication mechanism for programmatic access (passed as
`Authorization: Bearer <key>`).

### Database file permissions

Since SQLite stores everything in a single file, ensure the database file and
its directory have appropriate filesystem permissions. The WAL journal
(`travel.db-wal`) and shared memory (`travel.db-shm`) files are created
alongside the main database file.

### Registration gating

In production, consider disabling registration after creating your user(s) by
setting `REGISTRATION_ENABLED=false`. Registration is also gated on SMTP
configuration — without email verification, registration is blocked regardless
of this setting.

---

## Task Runner Reference

All tasks are defined in `.mise.toml` and run via `mise run <task>`.

### Build

| Task              | Description                               |
| ----------------- | ----------------------------------------- |
| `build` (alias: `b`) | Debug build                           |
| `build:release`   | Release build with LTO and stripping      |

### Run

| Task     | Description                                           |
| -------- | ----------------------------------------------------- |
| `dev`    | Start seed + server + worker together                 |
| `serve`  | Start server with cargo-watch auto-reload             |
| `worker` | Start background sync worker                          |
| `seed`   | Seed database with test user and sample journeys      |

### Database

| Task          | Description                                          |
| ------------- | ---------------------------------------------------- |
| `db:create`   | Create the SQLite database file                      |
| `db:migrate`  | Run pending migrations (depends on `db:create`)      |
| `db:reset`    | Drop, recreate, and migrate from scratch             |
| `db:prepare`  | Regenerate `.sqlx/` query cache (depends on `db:migrate`) |

### Quality

| Task              | Description                                    |
| ----------------- | ---------------------------------------------- |
| `lint`            | Clippy with all targets and pedantic warnings  |
| `lint:fix`        | Auto-fix clippy issues                         |
| `format` (alias: `f`) | Format Rust (cargo fmt) + JS/CSS (Prettier) |
| `format:check`    | Check formatting without modifying files       |
| `typecheck`       | TypeScript type-checking for JS files          |
| `check`           | Run lint + format:check + typecheck together   |
| `test` (alias: `t`) | Run tests with cargo-nextest                 |

### Examples

```bash
# Run a single test by name
mise run test -- -E 'test(name_contains_this)'

# Run tests matching a regex
mise run test -- -E 'test(/my_regex/)'

# Build with extra cargo flags
mise run build -- --verbose
```

---

## Troubleshooting

### Server fails to start with "invalid ENCRYPTION_KEY"

The key must be exactly 64 hexadecimal characters (representing 32 bytes). Check
for trailing whitespace or newlines in your `.env` file.

### "REGISTRATION_ENABLED is true but SMTP is not configured"

Registration requires working email verification. Either configure SMTP (all
four fields: `SMTP_HOST`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `EMAIL_FROM`) or
accept that registration will be disabled.

### Database locked errors

SQLite has limited concurrency. The application sets `busy_timeout=5000` to
retry on locks, but if you're running multiple server instances against the same
database file, you may still see lock contention. SQLite is designed for
single-server deployments — run one server and one worker per database file.

### Migrations fail on startup

Check that the `DATABASE_URL` path is writable and the parent directory exists.
For absolute paths, use `sqlite:/full/path/to/travel.db`.

### Sync jobs stay "pending"

The sync worker is a separate process. Make sure it's running (`mise run worker`
or `mise run dev`). Check its logs for errors — it logs at `info` level by
default.

### Rail/flight enrichment not working

Verify the API keys are set correctly. The worker logs which enrichment steps it
skips and why. Run with `RUST_LOG=debug,travel_mapper::worker=trace` for
detailed output.

### Build fails with sqlx errors

If you've changed SQL queries, regenerate the query cache:

```bash
mise run db:prepare
```

This requires a live database with up-to-date migrations. The generated
`.sqlx/*.json` files must be committed.

---

## Environment Variables Reference

| Variable                   | Required | Default              | Description                                                       |
| -------------------------- | -------- | -------------------- | ----------------------------------------------------------------- |
| `TRIPIT_CONSUMER_KEY`      | Yes      | —                    | TripIt API OAuth consumer key                                     |
| `TRIPIT_CONSUMER_SECRET`   | Yes      | —                    | TripIt API OAuth consumer secret                                  |
| `ENCRYPTION_KEY`           | Yes      | —                    | 32-byte hex key (64 hex chars) for AES-256-GCM encryption         |
| `DATABASE_URL`             | No       | `sqlite:travel.db`   | SQLite database URL                                               |
| `PORT`                     | No       | `3000`               | Server bind port                                                  |
| `REGISTRATION_ENABLED`     | No       | `true`               | Set to `false` to disable registration (also disabled without SMTP) |
| `SYNC_POLL_INTERVAL_SECS`  | No       | `5`                  | Sync worker poll interval in seconds                              |
| `AIRLABS_API_KEY`          | No       | —                    | AirLabs API key for flight status enrichment                      |
| `OPENSKY_CLIENT_ID`        | No       | —                    | OpenSky Network OAuth2 client ID for route verification           |
| `OPENSKY_CLIENT_SECRET`    | No       | —                    | OpenSky Network OAuth2 client secret                              |
| `DARWIN_API_TOKEN`          | No       | —                    | National Rail Darwin API token for UK rail status                 |
| `DB_RIS_API_KEY`           | No       | —                    | Deutsche Bahn RIS API key for German rail status                  |
| `DB_RIS_CLIENT_ID`         | No       | —                    | Deutsche Bahn RIS client ID                                       |
| `TRANSITLAND_API_KEY`      | No       | —                    | Transitland API key for global rail GTFS-RT status                |
| `ATTACHMENTS_PATH`         | No       | —                    | Filesystem directory for attachment storage; unset disables uploads |
| `SMTP_HOST`                | No       | —                    | SMTP server host for transactional email                          |
| `SMTP_PORT`                | No       | `587`                | SMTP server port                                                  |
| `SMTP_USERNAME`            | No       | —                    | SMTP auth username                                                |
| `SMTP_PASSWORD`            | No       | —                    | SMTP auth password                                                |
| `EMAIL_FROM`               | No       | —                    | From address for outgoing emails                                  |
| `VAPID_PRIVATE_KEY_PATH`   | No       | —                    | Path to PEM-encoded VAPID private key for Web Push                |
| `VAPID_PUBLIC_KEY`         | No       | —                    | Base64url-encoded VAPID public key served to browsers             |
| `RUST_LOG`                 | No       | `info,tower_http=debug` | tracing filter directive                                       |
| `TRIPIT_ACCESS_TOKEN`      | No       | —                    | Seed script only: pre-populate test user TripIt credentials       |
| `TRIPIT_ACCESS_TOKEN_SECRET` | No    | —                    | Seed script only: pre-populate test user TripIt credentials       |
