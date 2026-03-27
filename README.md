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

## Quick Start

```bash
git clone https://github.com/butlerx/travel-export
cd travel-export
mise install
cp .env.example .env    # fill in your TripIt keys and encryption key
mise run dev            # starts seed + server + worker
```

Visit `http://localhost:3000` and log in with `test` / `test`.

For detailed setup instructions, environment variables, optional integrations
(flight/rail status, email, push notifications), production deployment, and
troubleshooting, see the
**[Setup and Configuration Guide](docs/setup-and-configuration.md)**.

## API

Full API documentation is available via Swagger UI at `/docs` and as an OpenAPI
spec at `/openapi.json` when the server is running.

All authenticated endpoints accept either a session cookie (`session_id` from
login) or an API key (`Authorization: Bearer <key>` from `/auth/api-keys`).

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

## Configuration

See the **[Setup and Configuration Guide](docs/setup-and-configuration.md)**
for the full environment variables reference, optional integrations, database
management, SMTP, VAPID, and troubleshooting.
