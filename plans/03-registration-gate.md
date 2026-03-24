# 03 — Registration Gate

## What

Add an environment variable flag to disable open user registration. When disabled, the registration endpoint returns 403 and the UI hides the register link.

## Approach

1. Add `REGISTRATION_ENABLED` env var (default: `true`) — read at startup into `AppState`
2. Guard `POST /auth/register` handler: if disabled, return 403 with "Registration is currently disabled"
3. Guard `GET /register` page: if disabled, redirect to login or show a message
4. Conditionally hide "Register" link in navbar and login page based on state
5. Pass flag through Leptos context or as a prop to relevant components

## Files

- `src/server/state.rs` — add `registration_enabled: bool` to `AppState`
- `src/server.rs` or `src/bin/server.rs` — read env var at startup
- `src/server/routes/register.rs` — early return 403 when disabled
- `src/server/pages/register.rs` — redirect or show disabled message
- `src/server/components/navbar.rs` — conditionally render register link
- `src/server/pages/login.rs` — conditionally render "register" call-to-action

## Acceptance Criteria

- [ ] `REGISTRATION_ENABLED=false` → `POST /auth/register` returns 403
- [ ] `REGISTRATION_ENABLED=false` → `GET /register` shows disabled message or redirects
- [ ] `REGISTRATION_ENABLED=false` → navbar hides register link
- [ ] `REGISTRATION_ENABLED=true` (or unset) → existing behavior unchanged
- [ ] All existing tests pass
- [ ] `mise run lint` clean
