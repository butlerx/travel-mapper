# Travel Mapper

Sync your TripIt travel history to a local database and explore it through a web
dashboard, API, or CSV export for tools like Kepler.gl.

## Features

- **One-click TripIt sync** — connect your TripIt account and import all your
  trips
- **Web dashboard** — view your travel history, sync status, and travel map in
  the browser
- **Multiple export formats** — get your hops as JSON, CSV, or an HTML table
- **Multi-user** — each user connects their own TripIt account with isolated
  data
- **API keys** — generate keys for programmatic or scripted access
- **Background sync** — sync jobs run in the background so you're never waiting
- **Self-hosted** — runs on SQLite, no external database needed
- **Interactive API docs** — Swagger UI at `/docs` for exploring the API

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
DATABASE_URL=sqlite:travel.db
```

Generate an encryption key:

```bash
openssl rand -hex 32
```

Add it to `.env`:

```bash
ENCRYPTION_KEY=your_64_char_hex_key
```

### 4. Start the server and worker

```bash
mise run dev
```

The server starts on `http://localhost:3000` by default. The sync worker runs as
a separate process that polls for pending sync jobs.

### 5. Register and configure

_Note_: the dev env will be seeded with a `test` user with the password `test`

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

Connect your TripIt account via the OAuth flow:

1. Visit `http://localhost:3000/auth/tripit/connect` while logged in
2. Authorize the app on TripIt
3. You'll be redirected back and credentials are stored automatically

Or store TripIt OAuth tokens manually:

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

Full API documentation is available at `/docs` (Swagger UI) and `/openapi.json`.

### Public

#### Health Check

```
GET /health
```

Returns server status and last sync timestamp.

```json
{ "status": "ok", "last_sync": "2025-03-15 12:00:00" }
```

### Auth (public)

#### Register

```
POST /auth/register
```

```json
{ "username": "alice", "password": "secret" }
```

Returns `201 Created` with `{"id": 1, "username": "alice"}`.

#### Login

```
POST /auth/login
```

```json
{ "username": "alice", "password": "secret" }
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
{ "label": "my-key" }
```

Returns `{"id": 1, "key": "...", "label": "my-key"}`. The `key` value is only
shown once.

#### Store TripIt Credentials

```
PUT /auth/tripit
```

```json
{ "access_token": "...", "access_token_secret": "..." }
```

Stores your TripIt OAuth tokens encrypted at rest. Required before syncing.

#### TripIt OAuth Connect

```
GET /auth/tripit/connect
```

Starts the OAuth flow by redirecting to TripIt for authorization.

#### TripIt OAuth Callback

```
GET /auth/tripit/callback?oauth_token=...
```

Handles the redirect from TripIt, exchanges the request token for an access
token, and stores the credentials.

#### Sync Trips

```
POST /sync
```

Enqueues a sync job. If a sync worker is running, it will process the job in the
background. For browser requests, redirects back to the dashboard on completion.

```json
{ "trips_fetched": 42, "hops_fetched": 287, "duration_ms": 15230 }
```

#### Get Hops

```
GET /hops
GET /hops?type=air
```

Returns your travel hops. Optionally filter by type (`air`, `rail`, `cruise`,
`transport`).

Response format is determined by the `Accept` header:

| Accept Header                | Response Format |
| ---------------------------- | --------------- |
| `application/json` (default) | JSON array      |
| `text/csv`                   | CSV download    |
| `text/html`                  | HTML table      |

### Pages

The server also serves rendered HTML pages:

| Path         | Description                                    |
| ------------ | ---------------------------------------------- |
| `/`          | Landing page                                   |
| `/register`  | Registration form                              |
| `/login`     | Login form                                     |
| `/dashboard` | User dashboard with sync status and travel map |
| `/settings`  | Account settings                               |
| `/docs`      | Swagger UI for API documentation               |

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

| Variable                  | Required | Default            | Description                                    |
| ------------------------- | -------- | ------------------ | ---------------------------------------------- |
| `TRIPIT_CONSUMER_KEY`     | Yes      | --                 | Shared TripIt API consumer key                 |
| `TRIPIT_CONSUMER_SECRET`  | Yes      | --                 | Shared TripIt API consumer secret              |
| `ENCRYPTION_KEY`          | Yes      | --                 | 32-byte hex key (64 hex chars) for AES-256-GCM |
| `DATABASE_URL`            | No       | `sqlite:travel.db` | SQLite database URL                            |
| `PORT`                    | No       | `3000`             | Server port                                    |
| `SYNC_POLL_INTERVAL_SECS` | No       | `5`                | Sync worker poll interval in seconds           |
| `RUST_LOG`                | No       | --                 | Log level (e.g. `info`, `debug`)               |
