# 14 — Stats API Endpoint

## What

Move the `/stats` page handler into a proper API route at `/stats` that supports JSON, CSV, and HTML responses via the `Accept` header. The current handler lives in `pages/stats.rs` and only serves Leptos SSR HTML. After this work, API clients can fetch travel statistics as structured JSON while browsers still get the rendered page.

## Current Layout

| Route | Method | Purpose | Format |
|-------|--------|---------|--------|
| `/stats` | GET | Travel statistics page (Leptos SSR) | HTML only |

After this work, `GET /stats` serves JSON/CSV for API clients and the existing Leptos SSR page for `Accept: text/html`.

## Scope

### 1. Create `StatsResponse` API type

Define a `StatsResponse` struct in the new route module with `Serialize`, `Deserialize`, `Default`, `JsonSchema`:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct StatsResponse {
    pub total_journeys: usize,
    pub total_flights: usize,
    pub total_rail: usize,
    pub total_boat: usize,
    pub total_transport: usize,
    pub total_distance_km: u64,
    pub unique_airports: usize,
    pub unique_countries: usize,
    pub top_airlines: Vec<RankedItem>,
    pub top_aircraft: Vec<RankedItem>,
    pub top_routes: Vec<RankedItem>,
    pub cabin_class_breakdown: Vec<RankedItem>,
    pub seat_type_breakdown: Vec<RankedItem>,
    pub flight_reason_breakdown: Vec<RankedItem>,
    pub countries: Vec<RankedItem>,
    pub available_years: Vec<String>,
    pub selected_year: Option<String>,
    pub first_year: Option<String>,
    pub last_year: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RankedItem {
    pub name: String,
    pub count: usize,
}
```

### 2. Move handler to `routes/stats.rs`

- Create `src/server/routes/stats.rs` with a route handler that:
  1. Calls `negotiate_format(&headers)`
  2. Fetches data via `db::hops::GetAllForStats`
  3. Runs `compute_detailed_stats()` (keep this function in `pages/stats.rs` since it's shared with the page component)
  4. HTML → calls `pages::stats::render_page()` (extract from current `page()`)
  5. JSON → returns `StatsResponse` converted from `DetailedStats`
  6. CSV → returns CSV via `MultiFormatResponse`
- Implement `MultiFormatResponse` for `StatsResponse` (flat CSV with overview fields)
- Add `multi_format_docs!` for OpenAPI

### 3. Extract `render_page()` in `pages/stats.rs`

Split the current `page()` function:
- Extract the Leptos rendering into `pub fn render_page(stats: DetailedStats) -> String` (returns HTML string)
- Remove the current `page()` handler (replaced by the route handler)
- Keep `compute_detailed_stats()`, `DetailedStats`, `CountedItem`, and all Leptos components in `pages/stats.rs`

### 4. Wire up the route

- Add `pub(super) mod stats;` to `routes.rs`
- Register in `toplevel_api_routes()` or create a new `stats_api_routes()` + nest under `/stats`
- Remove `.route("/stats", get(stats::page))` from `page_routes()` in `pages.rs`

### 5. Implement `From<DetailedStats>` for `StatsResponse`

Map `CountedItem` → `RankedItem` and copy scalar fields. This keeps the internal computation type (`DetailedStats`) separate from the API type (`StatsResponse`).

## Approach

The stats page is read-only with no write endpoints — simpler than the journeys/trips migration. The main complexity is designing a good JSON shape for the nested breakdown fields. Using `RankedItem { name, count }` keeps it flat and queryable.

The `html_card` for `MultiFormatResponse` can render overview stat cards similar to the journey cards, or we can use the default key-value card for CSV/HTML since the primary HTML view is still the full Leptos page.

For CSV, flatten to one summary row per request (total_journeys, total_flights, etc.) since the data isn't a list of records. Alternatively, skip `MultiFormatResponse` entirely and handle JSON/HTML manually (JSON via `serde_json`, HTML via the existing page render). This avoids forcing tabular stats into a list-of-records CSV shape.

**Recommendation**: Don't implement `MultiFormatResponse` — stats are a single object, not a list. Instead:
- JSON → `Json(StatsResponse)`
- HTML → `pages::stats::render_page()`
- CSV → skip (stats aren't tabular) or serialize as single-row CSV

## Files

- `src/server/routes/stats.rs` — **new** route handler + `StatsResponse` type + OpenAPI docs
- `src/server/routes.rs` — add `pub(super) mod stats;`, register route
- `src/server/pages/stats.rs` — extract `render_page()`, keep computation + components
- `src/server/pages.rs` — remove `/stats` from `page_routes()`
- `src/server/state.rs` — add stats route to router

## Dependencies

- None — standalone change.

## Acceptance Criteria

- [ ] `GET /stats` with `Accept: application/json` returns structured JSON with all stat fields
- [ ] `GET /stats?year=2025` with `Accept: application/json` filters by year
- [ ] `GET /stats` with `Accept: text/html` returns the full SSR stats page (unchanged appearance)
- [ ] `/stats` appears in `/openapi.json` with proper schema
- [ ] `StatsQuery` year filter works for both JSON and HTML
- [ ] All existing stats tests pass (update URIs if needed)
- [ ] `mise run lint` clean
- [ ] `mise run test` green
