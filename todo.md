# Flighty Replacement — Migration Plan

> Goal: Replace personal Flighty usage with this self-hosted Travel Mapper app.

---

## Remaining Items

| # | Item | Plan |
|---|------|------|
| ~~1~~ | ~~[Multi-Content-Type Support](plans/01-multi-content-type.md)~~ | ✅ Done — `GET /journeys/{id}` serves JSON, CSV, and HTML via Accept header; separate page route removed |
| ~~2~~ | ~~[Rename hops → journeys (API + UI)](plans/02-rename-hops-journeys.md)~~ | ✅ Done — routes, UI, OpenAPI, JS, CSV headers, README all renamed |
| ~~3~~ | ~~[Registration Gate](plans/03-registration-gate.md)~~ | ✅ Done — `REGISTRATION_ENABLED` env var gates registration |
| 4 | [Email Validation](plans/04-email-validation.md) | Email field, verification flow, confirmation link |
| 5 | [PWA Push Notifications](plans/05-push-notifications.md) | Web Push API, subscription management, notify on events |
| 6 | [Travel Status Enrichment](plans/06-travel-status-enrichment.md) | Live/historical status for flights and trains |
| 7 | [Shareable Stats / Year in Review](plans/07-shareable-stats.md) | Public stats link, annual infographic |
| 8 | [Calendar / ICS Feed](plans/08-ics-feed.md) | Subscribable `.ics` endpoint for iCloud/Google Calendar |
| 9 | [Frequent Flyer / Miles Tracking](plans/09-miles-tracking.md) | Loyalty programs, miles calculation |
| 10 | [Photo Attachments](plans/10-photo-attachments.md) | File upload, gallery on detail page |
| 11 | [Cost Tracking](plans/11-cost-tracking.md) | Ticket prices, currency, spending analytics |
| 12 | [Carrier Icons](plans/12-carrier-icons.md) | Airline, train, ferry logos on journey list and detail pages |
| ~~13~~ | ~~[JS Modernization](plans/13-js-modernization.md)~~ | ✅ Done — ES Modules + JSDoc types, inline scripts extracted, XSS fixed, Prettier + tsc in mise |
