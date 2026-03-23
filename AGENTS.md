# AGENTS.md — Coding Agent Guide for travel_mapper

Rust 2024 edition / Axum web server / SQLite via sqlx / Leptos SSR components.
Task runner: [mise](https://mise.jdx.dev/). No Makefile, no CI workflows.

## Build / Run / Test Commands

All commands assume `mise install` has been run first.

```bash
# Build
mise run build            # debug build (alias: b)
mise run build:release    # release build (LTO + strip)

# Run
mise run serve            # cargo watch auto-reload server on :3000
mise run worker           # background sync worker
mise run dev              # seed + serve + worker together

# Database
mise run db:migrate       # create DB + run migrations
mise run db:reset         # drop, recreate, migrate
mise run db:prepare       # regenerate .sqlx/ query cache (REQUIRED after changing queries)
mise run seed             # create test user (test:test)

# Test
mise run test             # cargo nextest (alias: t)
mise run test -- -E 'test(name)' # run single test by name filter
mise run test -- -E 'test(/regex/)' # run tests matching regex

# Lint
mise run lint             # clippy --all-targets --all-features -- -D warnings
mise run lint:fix         # clippy --fix
mise run format           # cargo fmt (alias: f)
mise run format:check     # cargo fmt --check
mise run check            # lint + format:check together
```

### Running a single test

```bash
# By exact name
mise run test -- -E 'test(=insert_and_get_all_hops_roundtrip)'
# By substring
mise run test -- -E 'test(/hops/)'
# Or with cargo directly
cargo nextest run --all-targets --all-features -E 'test(/my_test_name/)'
```

### After changing SQL queries

Regenerate the offline query cache so compile-time checking works:

```bash
mise run db:prepare
```

This updates `.sqlx/*.json`. **Commit these files** alongside your query changes.

## Project Structure

```
src/
  lib.rs                            # top-level modules, #![warn(clippy::pedantic)]
  bin/
    server.rs                       # HTTP server entry point
    sync_worker.rs                  # background sync worker entry point
    seed.rs                         # DB seeding for development
  auth.rs                           # encryption, session helpers
  db.rs + db/                       # sqlx query objects (one file per table/command)
    api_keys.rs
    credentials.rs
    hops.rs + hops/                 # one file per query command
      create.rs, create_manual.rs, create_from_flighty.rs
      get_all.rs, get_all_for_stats.rs, get_by_id.rs
      delete_for_trip.rs, delete_stale.rs, replace_for_trip.rs
    oauth_tokens.rs
    sessions.rs
    sync_jobs.rs
    sync_state.rs
    users.rs
  geocode.rs + geocode/             # Nominatim geocoding + IATA airport lookup
    airports.rs                     # IATA code → coordinates
    nominatim.rs                    # OpenStreetMap geocoding client
    resolve.rs                      # coordinate resolution orchestration
    sanitize.rs                     # address string cleanup
  integrations.rs + integrations/   # third-party travel data sources
    flighty.rs                      # Flighty import
    tripit.rs + tripit/             # TripIt integration
      auth.rs                       # OAuth 1.0a signing
      fetch.rs + fetch/             # TripIt API data fetching
        client.rs                   # HTTP client + response handling
        parsers.rs                  # JSON → domain type parsing
        trips.rs                    # trip list + detail fetching
  server.rs + server/               # Axum web server
    components.rs + components/     # shared Leptos UI components
      auth_page.rs, error_page.rs, navbar.rs, shell.rs
    error.rs                        # error response types
    middleware.rs + middleware/
      auth.rs                       # AuthUser extractor
    pages.rs + pages/               # Leptos SSR page components
      add_flight.rs, landing.rs, login.rs, register.rs
      not_found.rs, unauthorized.rs, stats.rs
      dashboard.rs + dashboard/
        travel_stats.rs             # travel statistics sub-component
      hop_detail.rs + hop_detail/   # per-travel-type detail sections
        boat_section.rs, flight_section.rs
        rail_section.rs, transport_section.rs
      settings.rs + settings/       # settings page sections
        api_keys_section.rs, flighty_section.rs
        sync_section.rs, tripit_section.rs
    routes.rs + routes/             # one file per route handler
      api_keys.rs, flighty.rs, health.rs, hops.rs
      login.rs, logout.rs, register.rs
      static_assets.rs, sync.rs
      tripit_callback.rs, tripit_connect.rs, tripit_credentials.rs
    session.rs                      # session management
    state.rs                        # AppState definition
  worker.rs                         # background sync orchestration
  telemetry.rs                      # tracing/logging setup
migrations/                         # SQLite migrations (sqlx migrate)
static/                             # JS, CSS served at runtime
```

## Module Conventions

- **No `mod.rs` files.** Use `foo.rs` + `foo/` directory pattern (Rust 2018+ style).
- **No `helpers.rs` or `types.rs` files.** These are anti-patterns — place types alongside the code that uses them and co-locate helper functions with their callers.
- Parent module file declares `pub mod child;` for each submodule.
- Doc comments on every `pub mod` declaration:
  ```rust
  /// Query objects for the `hops` table — individual travel legs.
  pub mod hops;
  ```

## Import Ordering

Three groups separated by blank lines:

```rust
// 1. Local imports (super, crate)
use super::{ErrorResponse, MultiFormatResponse};
use crate::{
    db,
    server::{AppState, middleware::AuthUser},
};

// 2. External crates (grouped by crate, nested braces)
use aide::transform::TransformOperation;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
```

## Error Handling

**No `anyhow`.** Error strategy varies by layer:

| Layer | Pattern |
|-------|---------|
| DB | Return `Result<T, sqlx::Error>`. Propagate with `?`. |
| Domain | `#[derive(Debug, thiserror::Error)]` enums/structs. Wrap into `sqlx::Error::Decode(Box::new(...))` when needed. |
| Web helpers | Return `Result<T, (StatusCode, String)>` for direct HTTP mapping. |
| Route handlers | Return `Response` or `(CookieJar, Response)`. Convert errors via `ErrorResponse::into_format_response(msg, format, status)`. |

## DB / sqlx Patterns

- **Query objects**: small structs like `Create<'a>`, `GetAll`, `DeleteForTrip` with `pub async fn execute(&self, pool: &SqlitePool) -> Result<T, sqlx::Error>`.
- **Compile-time checked SQL**: `sqlx::query!()` and `sqlx::query_as!()` macros only.
- **Dynamic IN clauses**: use SQLite's `json_each()` table-valued function instead of `QueryBuilder`. Serialize IDs to a JSON array string and bind it as a single parameter:
  ```rust
  let ids_json = serde_json::to_string(&ids)
      .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
  sqlx::query!(
      "DELETE FROM hops WHERE user_id = ? AND trip_id NOT IN (SELECT value FROM json_each(?))",
      user_id,
      ids_json,
  )
  ```
- **PRAGMA exception**: SQLite PRAGMAs return untyped columns that `sqlx::query!()` cannot handle. These are the only queries allowed to use runtime `sqlx::query()`.
- **Row mapping**: internal `HopRow` struct matching query output, then `impl TryFrom<HopRow> for Row`.
- **Transactions**: `let mut tx = pool.begin().await?;` → queries on `&mut *tx` → `tx.commit().await?;`
- **Migrations**: sequential SQL files in `migrations/`. Never modify committed migrations.

## Axum Route Handler Patterns

- Extractors: `State<AppState>`, `AuthUser` (custom `FromRequestParts`), `CookieJar`, `HeaderMap`, `Bytes`.
- Content negotiation: `negotiate_format(&headers)` → `MultiFormatResponse` trait for JSON/CSV/HTML.
- Form + JSON: parse body based on `is_form_request(&headers)`, redirect on form success.
- OpenAPI: `aide` integration — each handler has a `*_docs` function for operation metadata.

## Type & Naming Conventions

- Types: `CamelCase` — `Row`, `TravelType`, `AuthUser`, `Geocoder`
- Functions: `snake_case` — `create_user_session`, `resolve_trip_coords`
- DB structs: short verb nouns — `Create`, `GetAll`, `GetByUserId`
- Error types: suffix with `Error` — `ParseTravelTypeError`, `AuthError`
- Enums: derive `Debug`, `Clone`, `PartialEq`, `Serialize`, `JsonSchema` as needed
- Use `#[must_use]` on pure helper functions

## Testing

- **Inline tests**: `#[cfg(test)] mod tests { ... }` in the same file — no separate test files.
- **Async**: all tests use `#[tokio::test]`.
- **Helpers** (in `src/db.rs` test module): `test_pool()` (in-memory SQLite), `test_user(&pool, name)`.
- **Server helpers** (in `src/server/test_helpers.rs`): `test_app_state`, `auth_cookie_for_user`, `body_text`, `sample_hop`.
- Tests run against in-memory SQLite — no external DB needed.

## Clippy & Formatting

- `#![warn(clippy::pedantic)]` is set in `lib.rs`. All pedantic lints are active.
- **Never use `#[allow(...)]`** to suppress clippy warnings. Fix the underlying code.
- **Never use `as any`, `@ts-ignore`-equivalents** — no type error suppression.
- No `rustfmt.toml` — default `cargo fmt` settings apply.
- Lint command treats warnings as errors: `-- -D warnings`.

## Logging

Use `tracing` with structured fields:

```rust
tracing::info!(user_id = auth.user_id, job_id, "sync job enqueued");
```

## Key Constraints

1. **Never modify committed migrations** — only safe to edit uncommitted ones.
2. **No `mod.rs` files** — use `foo.rs` + `foo/` pattern.
3. **No `#[allow(...)]`** — fix all clippy warnings at source.
4. **Coordinates are non-nullable** — `f64` everywhere, resolve via airport lookup or geocoding.
5. **Regenerate `.sqlx/`** after any SQL query change: `mise run db:prepare`.
6. **Feature flag `ssr`** gates Leptos server-side rendering — don't break SSR compilation.
