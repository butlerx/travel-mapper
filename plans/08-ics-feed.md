# 08 — Calendar / ICS Feed

## What

Serve a live `.ics` calendar feed that users can subscribe to from iCloud, Google Calendar, or any CalDAV client. The feed auto-updates as journeys are added or changed.

## Approach

1. **Per-user feed token**: Random token (separate from API keys) stored in DB; URL like `/feed/{token}.ics`
2. **ICS generation**: Use `icalendar` crate to build VCALENDAR with VEVENT entries per journey
3. **Event fields**: Summary (carrier + route), DTSTART/DTEND (departure/arrival), LOCATION (origin/destination airports/stations), DESCRIPTION (trip name, notes)
4. **No auth**: Token in URL is the access control (standard pattern for calendar subscriptions)
5. **Cache headers**: `Cache-Control` with short TTL so calendar apps poll for updates
6. **Settings UI**: Generate/revoke feed URL; copy-to-clipboard; show subscription instructions for iCloud/Google

## Dependencies

- Core data model (done)

## Files

- `Cargo.toml` — add `icalendar` crate
- `migrations/` — add `calendar_token` column to `users` or new `feed_tokens` table
- `src/db/` — token CRUD
- `src/server/routes/` — new `calendar_feed.rs` handler
- `src/server/state.rs` — register `/feed/{token}.ics` route
- `src/server/pages/settings.rs` — feed URL management UI

## Acceptance Criteria

- [ ] `GET /feed/{token}.ics` returns valid `text/calendar` content
- [ ] iCloud and Google Calendar can subscribe to the URL
- [ ] Events include departure/arrival times, locations, carrier info
- [ ] Feed updates automatically when journeys change
- [ ] Invalid/revoked tokens return 404
- [ ] User can generate and revoke feed URL from settings
- [ ] All existing tests pass
- [ ] `mise run lint` clean
