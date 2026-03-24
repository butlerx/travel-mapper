# 01 — Multi-Content-Type Support

## What

Ensure every API route supports JSON, HTML, and CSV responses via the `Accept` header using the existing `negotiate_format()` / `MultiFormatResponse` pattern. Merge the separate `/hop/{id}` HTML page route into `/hops/{id}` so that a single endpoint serves all formats.

## Current Route Layout

| Route | Method | Purpose | Format |
|-------|--------|---------|--------|
| `/hops` | GET | List hops | JSON, CSV, HTML (already negotiated) |
| `/hops` | POST | Create hop | JSON success or form redirect; negotiation only on errors |
| `/hops/{id}` | PUT | Update hop | JSON success or form redirect; negotiation only on errors |
| `/hops/{id}` | DELETE | Delete hop | JSON or redirect |
| `/hop/{id}` | GET | **Separate** HTML detail page (Leptos SSR) | HTML only |

After this work, `/hop/{id}` is eliminated. `GET /hops/{id}` serves the Leptos SSR detail page for `Accept: text/html` and JSON/CSV for API clients.

## Scope

### 1. Merge `/hop/{id}` into `GET /hops/{id}`

Currently the detail page lives at `/hop/{id}` (registered in `pages.rs`) and the API route for a single hop is only PUT/DELETE at `/hops/{id}`. We need to:

- Add a `GET /hops/{id}` handler that calls `negotiate_format`:
  - `text/html` → render the existing `hop_detail::page` Leptos component (same SSR output as current `/hop/{id}`)
  - `application/json` → return the hop as JSON
  - `text/csv` → return the hop as CSV
- Remove the `/hop/{id}` page route from `pages.rs`
- Update all internal links and redirects from `/hop/{id}` to `/hops/{id}`:
  - `src/server/routes/hops.rs` — redirect targets in create/update/delete handlers
  - `src/server/pages/hop_detail.rs` — any self-referencing URIs
  - `src/server/pages/trip_detail.rs` — hop links
  - `static/map.js` — JS links (`'/hop/' + id` → `'/hops/' + id`)

### 2. Multi-format for write endpoints

| Endpoint | Work Needed |
|----------|-------------|
| `POST /hops` (create) | Implement `MultiFormatResponse` for success; `negotiate_format` in success path |
| `PUT /hops/{id}` (update) | Implement `MultiFormatResponse` for success; `negotiate_format` in success path |
| `POST /import/csv` | Implement `MultiFormatResponse` for success; call `negotiate_format` in success path |

## Approach

### Merge
1. Add `get(get_hop_handler)` to the `/{id}` route in `hops_api_routes()` alongside the existing `put` and `delete`
2. `get_hop_handler` calls `negotiate_format(&headers)`:
   - HTML → call `hop_detail::page` (existing Leptos component) and return its response
   - JSON/CSV → fetch the hop from DB, return via `MultiFormatResponse`
3. Remove `.route("/hop/{id}", get(hop_detail::page))` from `page_routes()` in `pages.rs`
4. Find-and-replace all `/hop/{id}` references → `/hops/{id}` across routes, pages, and static JS

### Write endpoints
1. Add `impl MultiFormatResponse for {ResponseType}` with `build_csv()` and `build_html()` methods
2. Replace `Json(response)` success returns with `response.single_format_response(&format)`
3. Ensure `negotiate_format(&headers)` is called in the success path (not just error path)
4. Keep form submission path unchanged (still redirects)

## Files

- `src/server/routes/hops.rs` — add `get_hop_handler`; update create/update handlers; fix `/hop/` redirect targets
- `src/server/routes.rs` — add `get` to `/{id}` route; `MultiFormatResponse` trait (reference)
- `src/server/pages.rs` — remove `/hop/{id}` route
- `src/server/pages/hop_detail.rs` — update self-referencing URIs from `/hop/` to `/hops/`
- `src/server/pages/hop_detail/edit_form.rs` — form action already uses `/hops/{id}` (no change needed)
- `src/server/pages/trip_detail.rs` — update hop links from `/hop/{id}` to `/hops/{id}`
- `src/server/routes/csv_import.rs` — add multi-format to success path
- `static/map.js` — update JS links from `/hop/` to `/hops/`

## Dependencies

- None — this is a standalone change.

## Acceptance Criteria

- [ ] `GET /hops/{id}` with `Accept: text/html` returns the full SSR detail page
- [ ] `GET /hops/{id}` with `Accept: application/json` returns JSON
- [ ] `GET /hops/{id}` with `Accept: text/csv` returns CSV
- [ ] `/hop/{id}` no longer exists (returns 404)
- [ ] All internal links and redirects point to `/hops/{id}` (no remaining `/hop/` references)
- [ ] `curl -H "Accept: application/json" -X POST /hops ...` returns JSON
- [ ] `curl -H "Accept: text/csv" -X POST /hops ...` returns CSV
- [ ] `curl -H "Accept: text/html" -X POST /hops ...` returns HTML
- [ ] Same for `PUT /hops/{id}` and `POST /import/csv`
- [ ] Existing form submission flows still redirect correctly
- [ ] All existing tests pass
- [ ] `mise run lint` clean
