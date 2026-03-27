# Travel Mapper

Sync your TripIt travel history to a local database and explore it through a web
dashboard, API, or CSV export for tools like Kepler.gl.

## Features

- **One-click TripIt sync** — connect your TripIt account and import all your
  trips
- **CSV import** — import from Flighty, myFlightradar24, OpenFlights, and App in
  the Air
- **Trip grouping** — organise journeys into named trips with auto-grouping
- **Web dashboard** — view your travel history, sync status, and interactive
  travel map
- **Travel statistics** — aggregated stats page with distance, countries, and
  travel type breakdowns
- **Multiple export formats** — get your journeys as JSON, CSV, or an HTML table
- **Calendar feed** — subscribe to an ICS feed of your upcoming travel
- **Shareable stats** — generate a public link to share your travel stats
- **Photo attachments** — attach photos and documents to individual journeys
- **Push notifications** — opt-in Web Push alerts for sync completion and updates
- **Email verification** — verify your email address for account recovery
- **Multi-user** — each user connects their own TripIt account with isolated
  data
- **API keys** — generate keys for programmatic or scripted access
- **Background sync** — sync jobs run in the background so you're never waiting
- **Flight status enrichment** — live and historical flight status via AirLabs
  and OpenSky
- **Rail status enrichment** — Transitland GTFS-RT integration for rail journey
  updates
- **PWA support** — installable as a Progressive Web App with offline shell
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

#### Update Profile

```
PUT /auth/profile
```

```json
{ "first_name": "Alice", "last_name": "Smith" }
```

Updates the user's display name.

#### Update Email

```
PUT /auth/email
```

```json
{ "email": "alice@example.com" }
```

Sets or changes the user's email address and sends a verification link.

#### Resend Verification Email

```
POST /auth/resend-verification
```

Re-sends the verification email for the current user's unverified address.

#### Verify Email

```
GET /auth/verify-email?token=...
```

Confirms an email address using the token from the verification email.

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
{ "trips_fetched": 42, "journeys_fetched": 287, "duration_ms": 15230 }
```

#### Get Journeys

```
GET /journeys
GET /journeys?type=air
```

Returns your travel journeys. Optionally filter by type (`air`, `rail`, `boat`,
`transport`).

Response format is determined by the `Accept` header:

| Accept Header                | Response Format |
| ---------------------------- | --------------- |
| `application/json` (default) | JSON array      |
| `text/csv`                   | CSV download    |
| `text/html`                  | HTML table      |

#### Create Journey

```
POST /journeys
```

Manually create a journey with origin/destination, dates, and travel type.

#### Get / Update Journey

```
GET /journeys/{id}
PUT /journeys/{id}
```

Retrieve or update a specific journey's details (carrier, cost, loyalty, etc.).

#### Journey Attachments

```
GET  /journeys/{id}/attachments
POST /journeys/{id}/attachments
GET  /journeys/{id}/attachments/{attachment_id}
DELETE /journeys/{id}/attachments/{attachment_id}
```

Upload, list, retrieve, and delete photo/document attachments on a journey.
Requires `ATTACHMENTS_PATH` to be configured.

#### Trips

```
GET  /trips
POST /trips
GET  /trips/{id}
PUT  /trips/{id}
DELETE /trips/{id}
POST /trips/auto-group
```

CRUD operations on named trip groups. `auto-group` clusters unassigned journeys
into trips by date proximity.

#### Assign / Remove Journeys from Trips

```
POST   /trips/{id}/journeys
DELETE /trips/{id}/journeys/{journey_id}
```

Add or remove journeys from a trip.

#### CSV Import

```
POST /import/csv
```

Upload a CSV file to import journeys. Auto-detects format (Flighty,
myFlightradar24, OpenFlights, App in the Air).

#### Travel Statistics

```
GET /stats
```

Returns aggregated travel statistics (total distance, countries visited, travel
type breakdown). Supports JSON and HTML.

#### Settings

```
GET /settings
```

Account settings page with sections for profile, email, TripIt, sync, API keys,
CSV import, calendar feed, shareable stats, and push notifications.

#### Feed Tokens

```
POST   /auth/feed-tokens
DELETE /auth/feed-tokens/{id}
```

Create or revoke calendar feed access tokens.

#### Share Tokens

```
POST   /auth/share-tokens
DELETE /auth/share-tokens/{id}
```

Create or revoke shareable stats access tokens.

#### Push Notifications

```
POST   /auth/push-subscribe
DELETE /auth/push-subscribe
GET    /auth/vapid-public-key
```

Subscribe/unsubscribe from Web Push notifications and retrieve the VAPID public
key. Requires `VAPID_PRIVATE_KEY_PATH` and `VAPID_PUBLIC_KEY` to be configured.

### Public (token-authenticated)

#### Calendar Feed

```
GET /feed/{token}
```

Returns an ICS calendar feed of upcoming journeys. The token is created via
`/auth/feed-tokens`.

#### Shareable Stats

```
GET /share/{token}
```

Public stats page accessible via a share token. The token is created via
`/auth/share-tokens`.

### Pages

The server also serves rendered HTML pages:

| Path              | Description                                    |
| ----------------- | ---------------------------------------------- |
| `/`               | Landing page                                   |
| `/register`       | Registration form                              |
| `/login`          | Login form                                     |
| `/dashboard`      | User dashboard with sync status and travel map |
| `/journeys/new`   | Manual journey creation form                   |
| `/journeys/{id}`  | Journey detail with edit form and attachments  |
| `/trips`          | Trip list with grouping controls               |
| `/trips/{id}`     | Trip detail with assigned journeys             |
| `/stats`          | Aggregated travel statistics and map           |
| `/settings`       | Account settings (profile, email, sync, keys, import, feed, share, push) |
| `/share/{token}`  | Public shareable stats page                    |
| `/feed/{token}`   | ICS calendar feed (not HTML)                   |
| `/docs`           | Swagger UI for API documentation               |

## Visualising in Kepler.gl

1. Start the server, register, store credentials, and sync your trips
2. Download the CSV:
   ```bash
   curl -H "Authorization: Bearer <your-api-key>" \
     -H "Accept: text/csv" \
     http://localhost:3000/journeys -o travel_map.csv
   ```
3. Go to [kepler.gl/demo](https://kepler.gl/demo)
4. Drag and drop `travel_map.csv`
5. Add an **Arc Layer** with origin/destination coordinates
6. Colour by `travel_type`

## Environment Variables

| Variable                  | Required | Default            | Description                                                    |
| ------------------------- | -------- | ------------------ | -------------------------------------------------------------- |
| `TRIPIT_CONSUMER_KEY`     | Yes      | --                 | TripIt API OAuth consumer key                                  |
| `TRIPIT_CONSUMER_SECRET`  | Yes      | --                 | TripIt API OAuth consumer secret                               |
| `ENCRYPTION_KEY`          | Yes      | --                 | 32-byte hex key (64 hex chars) for AES-256-GCM encryption      |
| `DATABASE_URL`            | No       | `sqlite:travel.db` | SQLite database URL                                            |
| `PORT`                    | No       | `3000`             | Server bind port                                               |
| `REGISTRATION_ENABLED`    | No       | `true`             | Set to `false` to disable new user registration                |
| `SYNC_POLL_INTERVAL_SECS` | No       | `5`                | Sync worker poll interval in seconds                           |
| `AIRLABS_API_KEY`         | No       | --                 | AirLabs API key for flight status enrichment                   |
| `OPENSKY_CLIENT_ID`       | No       | --                 | OpenSky Network OAuth2 client ID for route verification        |
| `OPENSKY_CLIENT_SECRET`   | No       | --                 | OpenSky Network OAuth2 client secret                           |
| `ATTACHMENTS_PATH`        | No       | --                 | Filesystem directory for attachment storage; unset disables uploads |
| `SMTP_HOST`               | No       | --                 | SMTP server host for transactional email                       |
| `SMTP_PORT`               | No       | `587`              | SMTP server port                                               |
| `SMTP_USERNAME`           | No       | --                 | SMTP auth username                                             |
| `SMTP_PASSWORD`           | No       | --                 | SMTP auth password                                             |
| `EMAIL_FROM`              | No       | --                 | From address for outgoing emails                               |
| `VAPID_PRIVATE_KEY_PATH`  | No       | --                 | Path to PEM-encoded VAPID private key for Web Push             |
| `VAPID_PUBLIC_KEY`        | No       | --                 | Base64url-encoded VAPID public key served to browsers           |
| `RUST_LOG`                | No       | `info,tower_http=debug` | tracing filter directive                                  |

SMTP fields (`SMTP_HOST`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `EMAIL_FROM`) must
all be set to enable email verification. When SMTP is not configured, email
sending is silently skipped and registration does not require email verification.

VAPID fields (`VAPID_PRIVATE_KEY_PATH`, `VAPID_PUBLIC_KEY`) must both be set to
enable Web Push notifications. When absent, the push notification UI is hidden.
