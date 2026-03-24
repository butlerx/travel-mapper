# 07 — Shareable Stats / Year in Review

## What

Generate a public, shareable stats summary page — total distance, countries visited, journeys taken, a map — accessible via a unique URL without authentication.

## Approach

1. **Share token**: Per-user random token stored in DB; generates a public URL like `/share/{token}`
2. **Stats snapshot**: Render a read-only version of the stats page with the user's data
3. **Year filter**: Optional `?year=2025` param for annual review
4. **No auth required**: Public route, no session needed — token is the access control
5. **Generate/revoke**: Settings page toggle to create or revoke the share link
6. **OG meta tags**: Social-friendly meta tags for link previews

## Dependencies

- Stats page (done — item 5)
- Map rendering (done — item 9)

## Files

- `migrations/` — add `share_token` column to `users` table
- `src/db/users.rs` — token generation/lookup queries
- `src/server/routes/` — new `share.rs` handler
- `src/server/pages/` — new `shared_stats.rs` page (or reuse stats with public flag)
- `src/server/pages/settings.rs` — share link management UI
- `src/server/state.rs` — register share route
- `src/server/components/shell.rs` — OG meta tags for shared pages

## Acceptance Criteria

- [ ] User can generate a share link from settings
- [ ] `/share/{token}` renders stats without login
- [ ] Year filter works on shared page
- [ ] Share link can be revoked
- [ ] OG meta tags render for social previews
- [ ] Invalid/revoked tokens return 404
- [ ] All existing tests pass
- [ ] `mise run lint` clean
