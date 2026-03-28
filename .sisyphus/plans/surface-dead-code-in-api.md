# Plan: Surface Dead Code in API

> Status: **Not started**
> Created: 2026-03-28
> Context: Visibility audit + dead code cleanup revealed data that's parsed/stored but never exposed to users. This plan reintegrates it into the API.

## Overview

Six phases to expose enrichment data, reference lookups, and Transitland infrastructure to users. Phases 1–5 are independent; Phase 6 depends on Phase 5.

---

## Phase 1 — Enrichment Freshness

**Goal**: Restore TTL-based freshness logic so the worker skips recent enrichments, and surface staleness info in API responses.

### Tasks

1. **Restore deleted functions in `src/worker.rs`**:
   - `ENRICHMENT_TTL_SECS` (24h), `REALTIME_TTL_SECS` (2h)
   - `departure_aware_ttl(start_date)` — pick TTL based on whether departure is within 48h
   - `is_enrichment_fresh(pool, hop_id, provider, start_date)` — check `fetched_at` against TTL
2. **Wire freshness into enrichment functions**:
   - `enrich_flight_statuses`, `verify_flight_routes`, `enrich_rail_statuses` should skip hops with fresh enrichments
3. **Fix known gaps**:
   - **No-data sentinel**: When provider returns `Ok(None)`, write a row with `status = NULL` so freshness check works (currently causes repeated API calls)
   - **Date parsing**: `departure_aware_ttl` only parses `%Y-%m-%d`; handle ISO datetimes too
   - **Batch freshness**: Replace per-hop DB roundtrips with batch query via `GetByHopIdsAndProvider`
   - **AirLabs rate-limit**: Add `RateLimited` variant to `FlightStatusError` in `src/integrations/flight_status.rs`; detect 429 in `src/integrations/airlabs.rs`; worker should back off on rate limit
4. **Surface in API**: Add `fetched_at` (Option<String>) and `is_fresh` (bool) to `JourneyResponse` in `src/server/routes/journeys.rs`

### Key files

- `src/worker.rs`
- `src/db/status_enrichments.rs`
- `src/integrations/flight_status.rs`
- `src/integrations/airlabs.rs`
- `src/server/routes/journeys.rs`

### Removes `#[allow(dead_code)]`

- None directly (code was deleted, needs restoration from git history)

---

## Phase 2 — Enrichment Details Endpoint

**Goal**: New endpoint returning all enrichment rows for a journey, giving users full provider-level detail.

### Tasks

1. **New route**: `GET /journeys/{id}/enrichments`
2. **Response shape**: Array of enrichment objects per hop:
   - `hop_id`, `provider`, `status`, `delay_minutes`, `dep_gate`, `dep_terminal`, `arr_gate`, `arr_terminal`, `dep_platform`, `arr_platform`, `fetched_at`, `is_fresh`
   - Optionally include `raw_json` behind a query param (`?include_raw=true`)
3. **Follow existing patterns**: Use `MultiFormatResponse` for JSON/CSV/HTML content negotiation
4. **Auth**: Require `AuthUser` extractor
5. **OpenAPI**: Add `aide` docs function

### Key files

- New: `src/server/routes/enrichments.rs` (or extend `src/server/routes/journeys.rs`)
- `src/db/status_enrichments.rs` — existing `GetByHopIds` query is sufficient
- `src/server/routes.rs` — register new route

### Removes `#[allow(dead_code)]`

- None directly

---

## Phase 3 — Airport / Station Lookup Endpoints

**Goal**: Expose reference data so users (and future UI) can look up airports by IATA code and stations by name.

### Tasks

1. **`GET /api/airports/{iata}`**: Return full `Airport` struct (name, city, country, lat, lon, IATA, ICAO)
2. **`GET /api/stations/lookup?name={query}`**: Fuzzy CRS code lookup by station name
3. **`GET /api/stations/{crs}`**: Return station name for a CRS code
4. **Follow existing patterns**: `MultiFormatResponse`, `AuthUser` or public (decide based on use case)
5. **OpenAPI**: Add `aide` docs functions

### Key files

- `src/geocode/airports.rs` — remove `#[allow(dead_code)]` from `Airport` struct
- `src/geocode/stations.rs` — remove `#[allow(dead_code)]` from `lookup_crs`
- New: `src/server/routes/airports.rs`
- New: `src/server/routes/stations.rs`
- `src/server/routes.rs` — register new routes

### Removes `#[allow(dead_code)]`

- `Airport` struct in `src/geocode/airports.rs`
- `lookup_crs` in `src/geocode/stations.rs`

---

## Phase 4 — Provider Attribution

**Goal**: Show which provider supplied enrichment data on the journey detail page.

### Tasks

1. **Include provider name in journey detail response**: Use `provider_name()` trait method
2. **UI**: Show provider badge/label alongside status info on `journey_detail` page
3. **Remove dead_code markers** from trait method

### Key files

- `src/integrations/rail_status.rs` — remove `#[allow(dead_code)]` from `provider_name()` and `RailStatusQuery`
- `src/server/pages/journey_detail.rs` — render provider name
- `src/server/routes/journeys.rs` — include provider in response

### Removes `#[allow(dead_code)]`

- `RailStatusQuery` struct in `src/integrations/rail_status.rs`
- `provider_name()` method in `src/integrations/rail_status.rs`

---

## Phase 5 — Feed Discovery Endpoints

**Goal**: Restore deleted Transitland feed discovery code and expose it via API so users can see supported rail operators and their data feeds.

### Tasks

1. **Restore from git history** (deleted in dead code cleanup):
   - `discover_feeds_for_operator()` in `src/integrations/transitland/feed_discovery.rs`
   - `supported_operators()` in `src/integrations/transitland/feed_discovery.rs`
   - `TrenitaliaFrance` variant, `display_name()`, `country_code()` methods
2. **New routes**:
   - `GET /api/rail/operators` — list supported operators with metadata
   - `GET /api/rail/operators/{id}/feeds` — discover feeds for a specific operator
3. **Remove dead_code markers** from Transitland client types now used by these endpoints

### Key files

- `src/integrations/transitland/feed_discovery.rs` — restore deleted code
- `src/integrations/transitland/client.rs` — remove `#[allow(dead_code)]` from `RtFeedType`, `FeedSearchResponse`, `Feed`, `FeedUrls`, `ResponseMeta`
- New: `src/server/routes/rail.rs`
- `src/server/routes.rs` — register new routes

### Removes `#[allow(dead_code)]`

- `RtFeedType`, `FeedSearchResponse`, `Feed`, `FeedUrls`, `ResponseMeta` in `src/integrations/transitland/client.rs`

---

## Phase 6 — GTFS Match Transparency

**Goal**: Implement actual GTFS trip matching (previously a stub) and expose match results so users can see how their rail journeys map to GTFS data.

### Dependencies

- **Requires Phase 5** (feed discovery must be in place)

### Tasks

1. **Implement `match_journey_to_trip_id()`** in `src/integrations/transitland/matcher.rs` (was deleted as a stub — needs real implementation)
2. **Restore types**: `MatchError`, `TripCandidate`, `JourneyMatch` in matcher.rs
3. **Expose on journey detail**: Show stop/trip matching results (matched stops, confidence, GTFS trip ID)
4. **Remove dead_code markers** from cache types

### Key files

- `src/integrations/transitland/matcher.rs` — implement matching logic
- `src/integrations/transitland/cache.rs` — remove `#[allow(dead_code)]` from `StopMatch`, `TripMatch`
- `src/server/pages/journey_detail.rs` — render match info
- `src/server/routes/journeys.rs` — include match data in response

### Removes `#[allow(dead_code)]`

- `StopMatch`, `TripMatch` in `src/integrations/transitland/cache.rs`

---

## Items marked `#[allow(dead_code)]` (tracking)

| Item | File | Removed in Phase |
|------|------|-----------------|
| `Airport` struct | `src/geocode/airports.rs` | Phase 3 |
| `lookup_crs` function | `src/geocode/stations.rs` | Phase 3 |
| `RailStatusQuery` struct | `src/integrations/rail_status.rs` | Phase 4 |
| `provider_name()` method | `src/integrations/rail_status.rs` | Phase 4 |
| `RtFeedType` enum | `src/integrations/transitland/client.rs` | Phase 5 |
| `FeedSearchResponse` struct | `src/integrations/transitland/client.rs` | Phase 5 |
| `Feed` struct | `src/integrations/transitland/client.rs` | Phase 5 |
| `FeedUrls` struct | `src/integrations/transitland/client.rs` | Phase 5 |
| `ResponseMeta` struct | `src/integrations/transitland/client.rs` | Phase 5 |
| `StopMatch` struct | `src/integrations/transitland/cache.rs` | Phase 6 |
| `TripMatch` struct | `src/integrations/transitland/cache.rs` | Phase 6 |

## Code deleted in dead code cleanup (restore from git)

| Item | File | Restore in Phase |
|------|------|-----------------|
| `ENRICHMENT_TTL_SECS`, `REALTIME_TTL_SECS` | `src/worker.rs` | Phase 1 |
| `departure_aware_ttl()` | `src/worker.rs` | Phase 1 |
| `is_enrichment_fresh()` | `src/worker.rs` | Phase 1 |
| `get_flights_for_aircraft()` | `src/integrations/opensky.rs` | Not planned (truly unused) |
| `TrenitaliaFrance` variant | `src/integrations/transitland/feed_discovery.rs` | Phase 5 |
| `display_name()`, `country_code()` | `src/integrations/transitland/feed_discovery.rs` | Phase 5 |
| `discover_feeds_for_operator()` | `src/integrations/transitland/feed_discovery.rs` | Phase 5 |
| `supported_operators()` | `src/integrations/transitland/feed_discovery.rs` | Phase 5 |
| `InvalidTripDescriptor` variant | `src/integrations/transitland/gtfs_rt.rs` | Phase 6 |
| `MatchError`, `TripCandidate`, `JourneyMatch` | `src/integrations/transitland/matcher.rs` | Phase 6 |
| `match_journey_to_trip_id()` | `src/integrations/transitland/matcher.rs` | Phase 6 |
