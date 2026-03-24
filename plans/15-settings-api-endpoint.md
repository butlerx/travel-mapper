# 15 — Settings API Endpoint

## What

Move the `/settings` page handler into a proper API route that supports JSON and HTML responses via the `Accept` header. The current handler in `pages/settings.rs` only serves Leptos SSR HTML. After this work, API clients can fetch account settings state as structured JSON while browsers still get the rendered page.

## Current Layout

| Route | Method | Purpose | Format |
|-------|--------|---------|--------|
| `/settings` | GET | Settings page (Leptos SSR) | HTML only |

Supporting routes that already exist as API endpoints (no migration needed):
- `PUT /auth/tripit` — store TripIt credentials
- `GET /auth/tripit/connect` — start OAuth flow
- `GET /auth/tripit/callback` — OAuth callback
- `POST /auth/api-keys` — create API key
- `POST /sync` — trigger sync
- `POST /import/csv` — CSV import

After this work, `GET /settings` serves JSON for API clients and the existing Leptos SSR page for `Accept: text/html`.

## Scope

### 1. Create `SettingsResponse` API type

Define a response struct in the new route module:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SettingsResponse {
    pub tripit_connected: bool,
    pub sync: Option<SyncStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SyncStatus {
    pub status: String,
    pub last_sync_at: Option<String>,
    pub trips_fetched: i64,
    pub journeys_fetched: i64,
}
```

This is the read-only state that the settings page displays. Write operations (connect TripIt, trigger sync, create API key, import CSV) are already separate API endpoints.

### 2. Move handler to `routes/settings.rs`

- Create `src/server/routes/settings.rs` with a handler that:
  1. Calls `negotiate_format(&headers)`
  2. Queries `db::credentials::Has` and `db::sync_state::GetOrCreate`
  3. HTML → calls `pages::settings::render_page()` (extract from current `page()`)
  4. JSON → returns `SettingsResponse`
- Add `handler_docs` with OpenAPI metadata
- No CSV — settings aren't tabular data

### 3. Extract `render_page()` in `pages/settings.rs`

Split the current `page()` function:
- Extract the Leptos rendering into a `pub fn render_page(...)` that takes the data fields and `SettingsFeedback`, returns an HTML string
- Remove the current `page()` handler (replaced by the route handler)
- Keep the `Settings` component and all sub-components in `pages/settings/`
- Keep `SettingsFeedback` in `pages/settings.rs` (used by the render function)

### 4. Wire up the route

- Add `pub(super) mod settings;` to `routes.rs`
- Register in `toplevel_api_routes()` or nest under `/settings`
- Remove `.route("/settings", get(settings::page))` from `page_routes()` in `pages.rs`

### 5. Handle feedback query params

The HTML settings page uses query params (`?error=...`, `?tripit=connected`, `?csv=42`) for flash-message-style feedback after form actions redirect back. These only apply to the HTML response — JSON always returns current state without feedback messages.

The route handler should:
- HTML → pass `SettingsFeedback` through to `render_page()`
- JSON → ignore feedback params, return current state only

## Approach

Settings is read-only for the GET endpoint (all mutations happen via other endpoints). This is simpler than journeys/trips — no `MultiFormatResponse` needed since this is a single object, not a list.

Handle content negotiation manually:
- JSON → `Json(SettingsResponse)`
- HTML → `pages::settings::render_page(...)`
- CSV → return 406 Not Acceptable (settings aren't tabular)

### API keys in the response?

API keys are **not** included in `SettingsResponse` because:
1. They have their own `POST /auth/api-keys` endpoint
2. There's no existing `GET /auth/api-keys` list endpoint
3. Adding key listing is a separate concern

If listing API keys becomes needed, that should be a separate `GET /auth/api-keys` endpoint — not embedded in the settings response.

## Files

- `src/server/routes/settings.rs` — **new** route handler + `SettingsResponse` type + OpenAPI docs
- `src/server/routes.rs` — add `pub(super) mod settings;`, register route
- `src/server/pages/settings.rs` — extract `render_page()`, keep components
- `src/server/pages.rs` — remove `/settings` from `page_routes()`
- `src/server/state.rs` — add settings route to router

## Dependencies

- None — standalone change.

## Acceptance Criteria

- [ ] `GET /settings` with `Accept: application/json` returns `{"tripit_connected": bool, "sync": {...}}`
- [ ] `GET /settings` with `Accept: text/html` returns the full SSR settings page (unchanged appearance)
- [ ] JSON response ignores `?error=...` / `?tripit=...` / `?csv=...` query params
- [ ] HTML response still renders feedback alerts from query params
- [ ] `/settings` appears in `/openapi.json` with proper schema
- [ ] All existing settings tests pass (update handler paths if needed)
- [ ] `mise run lint` clean
- [ ] `mise run test` green
