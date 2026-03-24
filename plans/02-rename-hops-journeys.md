# 02 — Rename hops → journeys (API + UI Only)

## What

Rename all user-facing references from "hops" / "hop" to "journeys" / "journey". Internal Rust types, DB table names, and column names stay as `hops`.

## Scope

**Change** (user-facing):
- API route paths: `/hops` → `/journeys`, `/hops/new` → `/journeys/new`, `/hop/{id}` → `/journey/{id}`
- Navbar links and labels
- Page titles and headings ("Add Hop" → "Add Journey", etc.)
- OpenAPI/aide documentation strings
- Any redirect URLs pointing to old paths
- Trip detail sub-routes referencing hops (`/trips/{id}/hops` → `/trips/{id}/journeys`)
- Service worker cache paths if applicable

**Keep unchanged** (internal):
- DB table `hops` and all column names
- Rust types: `HopRow`, `CreateHop`, `GetAll`, etc.
- Internal variable names
- File names (`hops.rs`, `hop_detail.rs`, etc.) — optional, can discuss

## Approach

1. Update route registrations in `src/server/routes.rs` and `src/server/state.rs`
2. Update page route registrations in `src/server/pages.rs`
3. Update all Leptos component text (titles, headings, labels, links)
4. Update navbar items in `src/server/components/navbar.rs`
5. Update OpenAPI docs functions (`*_docs`)
6. Update redirect targets in route handlers
7. Consider adding redirect aliases from old `/hops` paths for backwards compatibility

## Files

- `src/server/routes.rs` — route builder functions, path constants
- `src/server/state.rs` — `.nest()` calls
- `src/server/pages.rs` — page route paths
- `src/server/routes/hops.rs` — redirect URLs in handlers
- `src/server/routes/trips.rs` — trip sub-routes referencing hops
- `src/server/pages/add_hop.rs` — page title, form labels
- `src/server/pages/hop_detail.rs` — page title, breadcrumbs
- `src/server/pages/dashboard.rs` — any hop references in UI
- `src/server/pages/trips.rs` — hop references in trip context
- `src/server/pages/trip_detail.rs` — hop references
- `src/server/components/navbar.rs` — nav link text and hrefs
- `static/sw.js` — cached route paths

## Acceptance Criteria

- [ ] All `/hops` routes respond at `/journeys` with same behavior
- [ ] All UI text says "journey"/"journeys" instead of "hop"/"hops"
- [ ] No user-visible string contains "hop" (except internal code)
- [ ] OpenAPI docs at `/docs` use "journey" terminology
- [ ] All existing tests pass (update test assertions as needed)
- [ ] `mise run lint` clean
