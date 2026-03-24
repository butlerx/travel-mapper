# 05 — PWA Push Notifications

## What

Add Web Push API support so PWA users can receive browser notifications for events like sync completion, flight status changes, and upcoming departures.

## Approach

1. **VAPID keys**: Generate server-side VAPID key pair; store public key in env/config
2. **Subscription storage**: New `push_subscriptions` table (user_id, endpoint, p256dh, auth, created_at)
3. **Client-side**: Extend `sw.js` with push event listener; add subscription UI in settings page
4. **Subscribe endpoint**: `POST /auth/push-subscribe` — store subscription
5. **Unsubscribe endpoint**: `DELETE /auth/push-subscribe` — remove subscription
6. **Server-side send**: Use `web-push` crate to send notifications
7. **Trigger points**: After sync completes (in worker), on flight status change (item 6), before upcoming departures (scheduled check)

## Dependencies

- Existing `sw.js` and PWA setup (done — `718f9c0`)
- Item 6 (Travel Status Enrichment) for status change notifications — can ship without, just won't have that trigger

## Files

- `migrations/` — new `push_subscriptions` table
- `src/db/` — new `push_subscriptions.rs`
- `src/server/routes/` — new push subscription handlers
- `src/server/state.rs` — register push routes
- `src/server/pages/settings.rs` — subscription toggle UI
- `static/sw.js` — add `push` and `notificationclick` event handlers
- `src/worker.rs` — trigger notification after sync
- `src/server.rs` — VAPID config

## Acceptance Criteria

- [ ] User can subscribe to push notifications from settings page
- [ ] `sw.js` handles `push` events and displays notifications
- [ ] Notification sent after sync completion
- [ ] Clicking notification opens relevant page
- [ ] User can unsubscribe
- [ ] Stale subscriptions (410 Gone) are cleaned up
- [ ] All existing tests pass
- [ ] `mise run lint` clean
