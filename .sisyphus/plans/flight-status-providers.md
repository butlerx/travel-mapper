# Plan: Replace AviationStack with AirLabs + OpenSky Network

## Goal

Replace the AviationStack flight status integration with a dual-provider approach:
- **AirLabs** (primary) — flight status, delays, gates, terminals by IATA flight number
- **OpenSky Network** (supplementary) — route verification, aircraft metadata, real-time position data

AirLabs covers all fields the current `FlightStatus` struct needs. OpenSky adds free data that AviationStack never provided: route verification (did the flight actually operate?) and aircraft type/registration.

## Current State

### Integration surface
- **Trait**: `FlightStatusApi` in `src/integrations/flight_status.rs` — single method `get_flight_status(flight_iata, flight_date) -> Result<Option<FlightStatus>>`
- **Client**: `AviationStackClient` implements the trait, hits `api.aviationstack.com/v1/flights`
- **Consumer**: `enrich_flight_statuses()` in `src/worker.rs` — called after each TripIt sync, iterates air-type hops, calls the trait, upserts results into `status_enrichments`
- **DB**: `status_enrichments` table with `UNIQUE(hop_id, provider)` — already supports multi-provider storage
- **Config**: `AVIATIONSTACK_API_KEY` env var flows through `server.rs` / `sync_worker.rs` → `AppState` / `SyncWorkerConfig`
- **UI**: Status badge + delay shown on journey detail page and list cards; gate/terminal shown in flight detail section

### Key observation
The DB schema already has a `provider` column with `UNIQUE(hop_id, provider)`, so storing enrichments from multiple providers is supported without migration changes. The `GetByHopIds` query returns the latest `fetched_at` per `hop_id` regardless of provider, which means the most recently fetched provider wins — acceptable behavior for now.

## Changes

### Phase 1: AirLabs client (direct AviationStack replacement)

#### 1.1 New file: `src/integrations/airlabs.rs`

Implement `FlightStatusApi` for an `AirLabsClient` struct.

**API details:**
- Base URL: `https://airlabs.co/api/v9`
- Endpoint: `/flight?flight_iata={code}&api_key={key}`
- Auth: API key as query parameter
- Returns single flight object (not wrapped in `data` array like AviationStack)

**Field mapping** (AirLabs response → existing `FlightStatus` struct):

| FlightStatus field | AirLabs JSON field | Notes |
|---|---|---|
| `flight_status` | `status` | Values: `scheduled`, `en-route`, `landed`, `cancelled` |
| `dep_delay_minutes` | `dep_delayed` | Minutes as integer, nullable |
| `arr_delay_minutes` | `arr_delayed` | Minutes as integer, nullable |
| `dep_gate` | `dep_gate` | String, nullable |
| `dep_terminal` | `dep_terminal` | String, nullable |
| `arr_gate` | `arr_gate` | String, nullable |
| `arr_terminal` | `arr_terminal` | String, nullable |
| `raw_json` | (full response) | Serialize entire response object |

**Differences from AviationStack to handle:**
- AirLabs returns a single object at the response root under a `response` key, not `data[0]`
- AirLabs uses `dep_delayed` / `arr_delayed` (not `departure.delay` / `arrival.delay`)
- AirLabs fields are flat (not nested under `departure` / `arrival` objects)
- AirLabs does not take a `flight_date` parameter on the `/flight` endpoint — it returns the closest matching flight. For date-specific lookup, use `/schedules?flight_iata={code}` and filter client-side by date, or accept closest-match behavior.

**Retry/error handling:** Replicate the existing retry logic from `AviationStackClient` (3 retries with exponential backoff on 5xx / 429 / connection errors).

**Date handling decision:** The `/flight` endpoint returns the closest flight for a given IATA code without date filtering. Two options:
- **Option A**: Use `/flight` endpoint and accept closest-match (simpler, works well for recent/active flights)
- **Option B**: Use `/schedules?flight_iata={code}` which returns multiple flights, filter by date client-side (more precise for historical enrichment)

Recommend **Option B** for accuracy — our sync enriches historical flights by date, so closest-match could return wrong day's data.

#### 1.2 Update `src/integrations/flight_status.rs`

- Keep the `FlightStatusApi` trait, `FlightStatus` struct, and `FlightStatusError` unchanged
- Remove `AviationStackClient` and its `impl FlightStatusApi` (or keep behind a feature flag if we want a fallback — recommend removing entirely to reduce dead code)
- Re-export from `src/integrations.rs`: add `pub mod airlabs;`

#### 1.3 Update `src/integrations.rs`

- Add `pub mod airlabs;` declaration
- Keep `pub mod flight_status;` (trait + types remain there)

#### 1.4 Update config: env vars and state

**Files:** `src/bin/server.rs`, `src/bin/sync_worker.rs`, `src/server/state.rs`, `src/worker.rs`

- Rename `AVIATIONSTACK_API_KEY` → `AIRLABS_API_KEY` in CLI arg definitions
- Rename `aviationstack_api_key` → `airlabs_api_key` in `AppState` and `SyncWorkerConfig`
- Update the doc comment on `AppState` field

#### 1.5 Update `src/worker.rs` — `enrich_flight_statuses()`

- Replace `AviationStackClient::new(api_key)` with `AirLabsClient::new(api_key)`
- Change provider string from `"aviationstack"` to `"airlabs"` in `Upsert` calls
- Update import: `use crate::integrations::airlabs::AirLabsClient;`
- Keep `use crate::integrations::flight_status::FlightStatusApi;`

#### 1.6 Update migration default

Add a new migration (e.g., `migrations/009_rename_provider_default.sql` — use next available number):
```sql
-- Update default provider from aviationstack to airlabs.
-- Existing rows keep their provider value unchanged.
ALTER TABLE status_enrichments ALTER COLUMN provider SET DEFAULT 'airlabs';
```

Note: SQLite does not support `ALTER COLUMN`. Instead, the default in the migration is cosmetic — the application code always provides the provider value explicitly. No migration needed; just ensure the code passes `"airlabs"` going forward. Existing `"aviationstack"` rows will still be readable and will be overwritten on next enrichment (same `hop_id`, different provider = new row; or if we want to replace, we could do a one-time `UPDATE status_enrichments SET provider = 'airlabs' WHERE provider = 'aviationstack'`).

**Decision**: Leave existing rows as-is. New enrichments write `"airlabs"`. The `GetByHopIds` query picks latest `fetched_at`, so new AirLabs data will supersede old AviationStack data naturally.

#### 1.7 Tests

- Add unit tests in `src/integrations/airlabs.rs` mirroring the existing AviationStack tests:
  - Happy path: mock server returns flight data → parsed correctly
  - Empty/not-found response → returns `None`
  - Retry on 5xx → succeeds after retries
- Update test provider strings in `src/db/status_enrichments.rs` tests from `"aviationstack"` to `"airlabs"` (or keep as-is since tests just need any valid string)

#### 1.8 Update docs

- Update `README.md` env var table: `AVIATIONSTACK_API_KEY` → `AIRLABS_API_KEY`
- Update `AGENTS.md` reference to "AviationStack flight status API client" → "AirLabs flight status API client"
- Update `.env.example` if it exists

### Phase 2: OpenSky Network client (supplementary enrichment)

#### 2.1 New file: `src/integrations/opensky.rs`

**Purpose**: Provide supplementary flight data that AirLabs doesn't offer for free:
- Route verification: confirm a flight actually operated between expected airports
- Aircraft metadata: type code, registration number

**API details:**
- Base URL: `https://opensky-network.org/api`
- Auth: OAuth2 client credentials flow (as of March 2026)
  - Token endpoint: `https://auth.opensky-network.org/auth/realms/opensky-network/protocol/openid-connect/token`
  - Grant type: `client_credentials`
  - Token expires in 30 minutes — cache and refresh
- Endpoint: `/flights/aircraft?icao24={hex}&begin={unix}&end={unix}`
  - Returns flights for a specific aircraft within a time window (max 2 days)
  - Response includes `estDepartureAirport`, `estArrivalAirport`, `callsign`, `firstSeen`, `lastSeen`

**Challenge**: OpenSky identifies aircraft by ICAO24 hex address, not by IATA flight number. To look up a flight, we need a mapping from airline IATA code → ICAO callsign prefix, then search by callsign. This is imperfect — callsigns don't always match IATA flight numbers.

**Practical approach**: Don't implement `FlightStatusApi` trait for OpenSky (it can't provide gate/terminal/delay data). Instead, create a separate trait or standalone functions:

```rust
pub struct OpenSkyClient { ... }

pub struct FlightVerification {
    pub operated: bool,
    pub est_departure_airport: Option<String>,  // ICAO code
    pub est_arrival_airport: Option<String>,     // ICAO code
    pub first_seen: Option<i64>,                 // unix timestamp
    pub last_seen: Option<i64>,                  // unix timestamp
    pub callsign: Option<String>,
}

pub struct AircraftInfo {
    pub icao24: String,
    pub registration: Option<String>,
    pub type_code: Option<String>,       // e.g., "A320", "B738"
    pub manufacturer: Option<String>,
    pub model: Option<String>,
}
```

**OAuth2 token management**: Implement token caching with expiry tracking. Use `reqwest` to POST to the token endpoint, cache the `access_token` and its expiry, refresh when within 60 seconds of expiry.

#### 2.2 Aircraft metadata via static database

OpenSky publishes their aircraft database as CSV snapshots at `https://opensky-network.org/datasets/metadata/`. Rather than downloading at runtime:
- Option A: Embed a subset as a static lookup table (aircraft type codes are relatively stable)
- Option B: Download CSV on first use, cache locally
- Option C: Skip aircraft metadata initially, add later if needed

**Recommend Option C** — keep Phase 2 scope focused on route verification. Aircraft metadata is a nice-to-have but not part of the current `FlightStatus` struct or UI.

#### 2.3 New DB column or table for verification data

OpenSky data doesn't fit the current `status_enrichments` schema (no gate/terminal/delay). Two options:
- **Option A**: Store as a separate provider row in `status_enrichments` with empty gate/terminal fields and route data in `raw_json`
- **Option B**: New table `flight_verifications` with specific columns

**Recommend Option A** for now — the `raw_json` column can hold the OpenSky response, and the `status` field can indicate `"verified"` / `"unverified"`. This avoids a new migration and keeps the query surface simple. The UI can parse `raw_json` if we want to display verification details later.

#### 2.4 Config: env vars

Add to `src/bin/server.rs`, `src/bin/sync_worker.rs`, `AppState`, `SyncWorkerConfig`:
- `OPENSKY_CLIENT_ID` — OAuth2 client ID (optional, OpenSky enrichment disabled if absent)
- `OPENSKY_CLIENT_SECRET` — OAuth2 client secret

#### 2.5 Update `src/worker.rs` — add OpenSky enrichment step

After AirLabs enrichment in `process_sync_job`:
```
enrich_flight_statuses(config, user_id).await;      // AirLabs (existing call)
verify_flight_routes(config, user_id).await;         // OpenSky (new call)
```

The new `verify_flight_routes` function:
- Check `config.opensky_client_id` is present, otherwise skip
- Fetch air-type hops
- For each hop with a flight number and date:
  - Map IATA flight number to a callsign (best-effort: airline ICAO prefix + flight number digits)
  - Query OpenSky `/flights/all?begin={start_of_day}&end={end_of_day}` filtered to matching callsign
  - If a matching flight is found, upsert to `status_enrichments` with provider `"opensky"`, status `"verified"`, and the route data in `raw_json`

**Rate limit awareness**: OpenSky gives 4,000 credits/day for authenticated users. Each flight lookup costs ~1 credit. With typical travel patterns (a few flights per sync), this is well within limits. Add a counter and stop enrichment if approaching the daily limit.

#### 2.6 Tests

- Unit tests for OAuth2 token acquisition (mock token endpoint)
- Unit tests for flight lookup parsing
- Unit tests for callsign mapping logic

### Phase 3: UI enhancements (optional, not blocking)

If we want to surface OpenSky data in the UI:
- Show a "verified" badge on flights where OpenSky confirmed the route
- Display aircraft type/registration if available from OpenSky's `raw_json`
- Show actual departure/arrival airports from OpenSky alongside the expected ones

This phase is purely additive and doesn't block the provider switch.

## Execution Order

1. **Phase 1.1–1.3**: Implement AirLabs client + update module structure
2. **Phase 1.4–1.5**: Update config and worker to use AirLabs
3. **Phase 1.6–1.7**: Handle migration concerns + add tests
4. **Phase 1.8**: Update docs
5. **Phase 2.1–2.3**: Implement OpenSky client + verification logic
6. **Phase 2.4–2.6**: Wire OpenSky into worker + add tests
7. **Phase 3**: UI enhancements (separate PR)

Phase 1 is a clean swap — the app works identically to before, just with AirLabs instead of AviationStack. Phase 2 adds new capabilities. Phase 3 is cosmetic.

## Risk Assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| AirLabs `/flight` endpoint returns wrong day's flight | Medium | Use `/schedules` endpoint + client-side date filter |
| AirLabs free tier (1,000 calls/mo) insufficient | Low | Personal project with few syncs; paid tier is $49/mo if needed |
| OpenSky callsign ≠ IATA flight number | Medium | Best-effort mapping; verification is supplementary, not critical |
| OpenSky OAuth2 token refresh complexity | Low | Standard OAuth2 client_credentials flow; cache token with expiry |
| Existing `"aviationstack"` rows in DB | None | Left as-is; new data supersedes via `fetched_at` ordering |

## Files Changed Summary

### Phase 1 (must change)
- `src/integrations/airlabs.rs` — **new file**
- `src/integrations/flight_status.rs` — remove `AviationStackClient`, keep trait + types
- `src/integrations.rs` — add `pub mod airlabs;`
- `src/worker.rs` — swap client, update provider string and imports
- `src/server/state.rs` — rename `aviationstack_api_key` → `airlabs_api_key`
- `src/bin/server.rs` — rename env var
- `src/bin/sync_worker.rs` — rename env var
- `README.md` — update env var docs
- `AGENTS.md` — update integration description

### Phase 2 (additive)
- `src/integrations/opensky.rs` — **new file**
- `src/integrations.rs` — add `pub mod opensky;`
- `src/worker.rs` — add `verify_flight_routes()` function
- `src/server/state.rs` — add OpenSky config fields
- `src/bin/server.rs` — add OpenSky env vars
- `src/bin/sync_worker.rs` — add OpenSky env vars
