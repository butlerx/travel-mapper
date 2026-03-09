# TripIt Travel Server

Multi-user web server that syncs TripIt travel history to a local SQLite database
and serves it via JSON, CSV, and HTML APIs.

## Features

- Multi-user support with registration and login
- Dual auth: session cookies (browser) + API keys (programmatic)
- Per-user TripIt OAuth credentials, encrypted at rest with AES-256-GCM
- Content negotiation via `Accept` header: JSON, CSV, HTML
- HTML responses rendered with Maud templates
- Per-user sync and hop isolation
- Compile-time checked SQL queries via `sqlx::query!` macros
- SQLite with WAL mode
- Retry with exponential backoff for TripIt API calls
- Graceful shutdown on Ctrl+C

## Requirements

- [mise](https://mise.jdx.dev/) for tool and task management
- A [TripIt Developer account](https://www.tripit.com/developer)

## Setup

### 1. Clone and install

```bash
git clone https://github.com/butlerx/travel-export
cd travel-export
mise install
mise run build
```

### 2. Register a TripIt app

1. Go to [tripit.com/developer](https://www.tripit.com/developer)
2. Create a new app (name/description can be anything)
3. Copy your **Consumer Key** and **Consumer Secret**

### 3. Configure environment

```bash
cp .env.example .env
```

Fill in your credentials in `.env`:

```bash
TRIPIT_CONSUMER_KEY=your_consumer_key
TRIPIT_CONSUMER_SECRET=your_consumer_secret
ENCRYPTION_KEY=your_64_char_hex_key
DATABASE_URL=sqlite:travel.db
PORT=3000
```

Generate an encryption key:

```bash
openssl rand -hex 32
```

### 4. Start the server

```bash
mise run serve
```

Or run directly:

```bash
cargo run --bin server
```

The server starts on `http://localhost:3000` by default.

### 5. Register and configure

Register a user:

```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret"}'
```

Login (returns a session cookie):

```bash
curl -c cookies.txt -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret"}'
```

Create an API key for programmatic access:

```bash
curl -b cookies.txt -X POST http://localhost:3000/auth/api-keys \
  -H "Content-Type: application/json" \
  -d '{"label":"cli"}'
```

Store your TripIt OAuth tokens (obtained from TripIt's OAuth flow):

```bash
curl -b cookies.txt -X PUT http://localhost:3000/auth/tripit \
  -H "Content-Type: application/json" \
  -d '{"access_token":"your_token","access_token_secret":"your_secret"}'
```

Trigger a sync:

```bash
curl -b cookies.txt -X POST http://localhost:3000/sync
```

## API Endpoints

### Public

#### Health Check

```
GET /health
```

Returns server status and last sync timestamp.

```json
{"status": "ok", "last_sync": "2025-03-15 12:00:00"}
```

### Auth (public)

#### Register

```
POST /auth/register
```

```json
{"username": "alice", "password": "secret"}
```

Returns `201 Created` with `{"id": 1, "username": "alice"}`.

#### Login

```
POST /auth/login
```

```json
{"username": "alice", "password": "secret"}
```

Returns a `session_id` cookie and `{"id": 1, "username": "alice"}`.

### Authenticated

All authenticated endpoints accept either:
- **Session cookie**: `Cookie: session_id=<token>` (from login)
- **API key**: `Authorization: Bearer <api-key>` (from `/auth/api-keys`)

#### Logout

```
POST /auth/logout
```

Clears the session.

#### Create API Key

```
POST /auth/api-keys
```

```json
{"label": "my-key"}
```

Returns `{"id": 1, "key": "...", "label": "my-key"}`. The `key` value is only shown once.

#### Store TripIt Credentials

```
PUT /auth/tripit
```

```json
{"access_token": "...", "access_token_secret": "..."}
```

Stores your TripIt OAuth tokens encrypted at rest. Required before syncing.

#### Sync Trips

```
POST /sync
```

Triggers a full sync from TripIt using your stored credentials.

```json
{"trips_fetched": 42, "hops_fetched": 287, "duration_ms": 15230}
```

#### Get Hops

```
GET /hops
GET /hops?type=air
```

Returns your travel hops. Optionally filter by type (`air`, `rail`, `cruise`, `transport`).

Response format is determined by the `Accept` header:

| Accept Header | Response Format |
|---|---|
| `application/json` (default) | JSON array |
| `text/csv` | CSV download |
| `text/html` | HTML table |

## Visualising in Kepler.gl

1. Start the server, register, store credentials, and sync your trips
2. Download the CSV:
   ```bash
   curl -H "Authorization: Bearer <your-api-key>" \
     -H "Accept: text/csv" \
     http://localhost:3000/hops -o travel_map.csv
   ```
3. Go to [kepler.gl/demo](https://kepler.gl/demo)
4. Drag and drop `travel_map.csv`
5. Add an **Arc Layer** with origin/destination coordinates
6. Colour by `travel_type`

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `TRIPIT_CONSUMER_KEY` | Yes | -- | Shared TripIt API consumer key |
| `TRIPIT_CONSUMER_SECRET` | Yes | -- | Shared TripIt API consumer secret |
| `ENCRYPTION_KEY` | Yes | -- | 32-byte hex key (64 hex chars) for AES-256-GCM |
| `DATABASE_URL` | No | `sqlite:travel.db` | SQLite database URL |
| `PORT` | No | `3000` | Server port |
| `RUST_LOG` | No | -- | Log level (e.g. `info`, `debug`) |

## Project Structure

```
travel-export/
├── .env                     # Your credentials (never commit)
├── .env.example             # Credentials template
├── .mise.toml               # Tool versions and tasks
├── Cargo.toml               # Project metadata and dependencies
├── migrations/
│   ├── 001_initial.sql      # Base schema (segments, sync_state)
│   ├── 002_multi_user.sql   # Multi-user schema (users, sessions, api_keys, credentials)
│   ├── 003_oauth_request_tokens.sql  # OAuth request token storage
│   ├── 004_sync_jobs.sql    # Background sync job queue
│   └── 005_rename_segments_to_hops.sql  # Rename segments → hops
└── src/
    ├── lib.rs               # Library root (clippy::pedantic enabled)
    ├── models.rs            # Travel hop types
    ├── db.rs                # SQLite access layer (compile-time checked queries)
    ├── auth.rs              # Auth module root
    ├── auth/
    │   ├── crypto.rs        # AES-256-GCM encrypt/decrypt
    │   ├── handlers.rs      # Register, login, logout, API keys, TripIt credentials
    │   ├── middleware.rs     # AuthUser extractor (session cookie + API key)
    │   └── password.rs      # Argon2 password hashing
    ├── routes.rs            # Routes module root
    ├── routes/
    │   ├── handlers.rs      # Health, hops, sync handlers
    │   ├── pages.rs         # Dashboard and landing page templates
    │   └── response.rs      # Content negotiation (JSON/CSV/HTML)
    ├── server.rs            # Server module root
    ├── server/
    │   ├── state.rs         # AppState, router setup
    │   └── sync.rs          # Per-user TripIt sync orchestration
    ├── tripit.rs            # TripIt module root
    ├── tripit/
    │   ├── auth.rs          # OAuth 1.0 HMAC-SHA1 signing
    │   └── fetch.rs         # TripIt API client + response parsers
    └── bin/
        └── server.rs        # Server binary entry point
```

## License

MIT
