# Flighty Feature-Parity Plan

A roadmap for closing the gap between **travel-export** (self-hosted multi-modal travel
logbook + analytics) and **Flighty** (native iOS real-time flight-day tracker).

## Positioning

These are two different products:

- **travel-export** — self-hosted Rust/Axum/SQLite/Leptos PWA. Strengths: multi-modal
  logbook (flight/rail/boat/transport), rich lifetime stats, multiple import sources,
  public sharing, REST API, data ownership.
- **Flighty** — cloud iOS app monetised via Pro. Strength: real-time *flight-day*
  intelligence (predictive delays, fast alerts, delay reasons), backed by pilot-grade
  feeds (FAA SWIM, Eurocontrol) and deep native OS integration (Live Activities, CarPlay).

**Strategy:** we already match or beat Flighty on the logbook/stats/multi-modal side.
The gap is the *live flight-day experience* — prediction, speed, and presence on the day
of travel. We close that gap where a self-hosted web app realistically can, and
explicitly decline the native/data-moat features that don't fit.

## Status

**Shipped:** the entire design track (P0 + P1 + P2, see [Design & Layout](#design--layout))
and six roadmap features — **1.1** push-on-status-delta, **1.2** Upcoming page + live poll,
**2.1** Year-in-Review share view, **2.2** email-forward import, **2.3** ICS calendar ingest,
**3.1** live single-flight sharing. **1.3** (inbound-aircraft prediction) has its foundation
in place (tail-number capture) but the live feed lookup is deferred pending an AirLabs key.
The map-consolidation prerequisite + Tier A "dark everywhere" are done.

**Remaining:** 1.3 live prediction lookup, 3.2 delay reasons/trend.

_Whole suite green: 387 tests passing; clippy / `cargo fmt` / `tsc` clean._

## Current parity snapshot

| Capability                                   | travel-export | Flighty        |
| -------------------------------------------- | ------------- | -------------- |
| Flight history logbook                       | ✅            | ✅ (Passport)  |
| Lifetime stats (distance/countries/airlines) | ✅ richer     | ✅             |
| Global map of trips                          | ✅            | ✅             |
| TripIt import                                | ✅            | ✅             |
| CSV import (Flighty/FR24/OpenFlights/AITA)   | ✅            | ❌             |
| Multi-modal: rail / boat / ground transport  | ✅            | ❌ flights only |
| Public shareable stats                       | ✅            | partial        |
| Calendar (ICS) feed                          | ✅ in + out   | ✅ input       |
| REST API + OpenAPI                            | ✅            | ❌             |
| Self-hosted / own your data                  | ✅            | ❌ cloud only  |
| Cost / loyalty miles tracking                | ✅            | ❌             |
| Attachments (photos/docs)                    | ✅            | ❌             |
| **Predictive inbound-aircraft delays**       | 🟡 tail-no.   | ✅             |
| **Fast push on status delta**                | ✅            | ✅             |
| **Delay reasons + trend (METAR/TAF)**        | ❌            | ✅             |
| **Day-of timeline / live flight view**       | partial (web) | ✅ Live Activity|
| **Email / calendar auto-import**             | ✅            | ✅             |
| **Friends / live single-flight sharing**     | ✅            | ✅             |
| Year-in-review recap                          | ✅            | ✅             |

## Roadmap

### Phase 1 — Flight-day core (highest leverage, reuses existing infra)

#### 1.1 Push on status delta (not on sync completion)

**Problem:** push notifications currently fire only on sync-job completion
(`src/server/push.rs`, driven from `src/worker.rs`). Flighty's entire value is alerts
*faster than the airline*.

**Change:** when the enrichment worker writes a new `status_enrichments` row, diff it
against the prior row for that hop and emit a push on meaningful transitions:

- delay started / delay increased
- gate or terminal changed
- cancelled / diverted
- boarding / departed / landed

**Touch points:** `src/worker.rs`, `src/db/status_enrichments.rs` (need a "latest for
hop" query + prior-value comparison), `src/server/push.rs`, `src/db/push_subscriptions.rs`.

**Effort:** Medium. **Impact:** High. Reuses all existing push + enrichment infra.

**✅ Done.** `StatusSnapshot::describe_transition` (in `db/status_enrichments.rs`) diffs the
incoming enrichment against the previously stored row and yields a short human string —
`Cancelled` / `Diverted` (reported alone), `Departed` / `Landed` / `Boarding`,
`Delayed Nm` / `Delay increased to Nm`, and gate/terminal/platform changes (combined with
` · `). The worker reads the prior row before each upsert (flight + rail) and calls
`push_status_transition`, which fires a Web Push via the existing `send_to_user`. Guards:
no push on initial data or from a no-data sentinel, and only within a notify window of the
travel day (`within_notify_window`: dep − 1 … dep + 2 days) so historical journeys never
notify. Covered by 11 unit tests.

#### 1.2 Upcoming flights page + live single-flight view

**Problem:** there is no day-of view — only the dashboard map + journey list.

**Change:**

- New "Upcoming" page: journeys in the near future, sorted by departure, showing
  countdown, gate, terminal, boarding time from `status_enrichments`.
- New live single-flight view that auto-refreshes (client-side poll) for an in-progress
  flight.

**Touch points:** new `src/server/pages/upcoming.rs` + route in `src/server/routes/`,
new `static/js/` poll script (per the no-inline-JS rule, add `include_str!` +
`js_handler!` in `static_assets.rs`), CSS partial under `static/css/`.

**Effort:** Medium. **Impact:** High.

**✅ Done (upcoming page + live poll).** `src/server/pages/upcoming.rs` (route `/upcoming`,
nav link added) lists journeys departing today or later, soonest first, each with a relative
countdown (Today / Tomorrow / In N days), status badge, delay, and gate/terminal/platform
chips drawn from the latest enrichment (batched via `GetByHopIds`). Same-day cards carry
`data-live="1"`; `static/js/upcoming-poll.js` polls the existing
`GET /journeys/{id}/enrichments` JSON endpoint every 60s and refreshes the status badge in
place. Styling in `static/css/upcoming.css`. Covered by page tests (auth, future-only,
empty state) + a static-asset test. A dedicated full-screen single-flight live view is not
yet built — the day-of need is met by the Upcoming list + in-place poll for now.

#### 1.3 Inbound-aircraft delay prediction ("where's my plane")

**Problem:** enrichment is after-the-fact only.

**Change:** key off tail number / registration in `status_enrichments`. Given the
aircraft's prior leg (via AirLabs / OpenSky), surface predicted knock-on delay before the
airline announces. Even a basic "your aircraft is currently delayed inbound" is a win.

**Touch points:** `src/integrations/airlabs.rs`, `src/integrations/opensky.rs`,
`src/integrations/flight_status.rs` (trait), `src/worker.rs`.

**Effort:** High (data modelling + feed reliability). **Impact:** High — this is
Flighty's signature feature.

**🟡 Foundation shipped; live lookup deferred.** Done: the keying mechanism is in place —
migration `019_aircraft_registration.sql` adds `status_enrichments.aircraft_reg`, AirLabs
parsing captures `reg_number` into `FlightStatus`, the worker persists it on every flight
enrichment, and it's surfaced through `EnrichmentResponse` / `JourneyResponse` (JSON + CSV)
and on the journey-detail page ("Aircraft EI-DEG"). Tail number is now tracked per leg.

Deferred (needs a live AirLabs key to validate response shapes before shipping): the actual
inbound prediction — querying the same registration's *current* leg, comparing its arrival
estimate against this flight's scheduled departure minus turnaround, and pushing "your
aircraft is delayed inbound (~N min)". The prediction model and the by-registration lookup
should be built and validated against the real feed rather than coded blind against guessed
JSON; the registration capture above is the prerequisite that unblocks it.

### Phase 2 — Auto-import & recap (cheap wins, marketable)

#### 2.1 Year-in-review share view

**Change:** repackage existing per-year stats into a shareable "Year in Review" view.
We already have share tokens (`src/db/share_tokens.rs`) and per-year stat filters
(`src/server/pages/stats.rs`) — this is mostly a new presentation + share flow.

**Effort:** Low. **Impact:** Medium (marketing/demo value).

**✅ Done.** New public route `GET /share/{token}/review` (`review_handler` in
`routes/share.rs`) renders `src/server/pages/year_in_review.rs` — a no-navbar recap with OG
meta: hero tiles (journeys / distance / countries), per-mode counts, highlight rows (top
airline / route / aircraft / most-visited country, plus a "N.N× around the world" distance
fact), and a year selector across all years behind the token. Defaults to the most recent
year with data; `?year=` selects a specific year (reuses `compute_detailed_stats`). The
shared-stats page links into it via a banner. Styling in `static/css/year-in-review.css`.
Covered by 3 route tests (latest-year default, explicit year, invalid token 404).

#### 2.2 Email-forward import

**Change:** a forward-to address that parses booking confirmation emails into journeys,
matching Flighty's AI email pipeline. Builds on existing email infra
(`src/server/email.rs`) and CSV/manual journey creation (`src/db/hops/create_manual.rs`).

**Effort:** High (parser reliability). **Impact:** Medium.

**✅ Done.** Three-tier extraction pipeline in `integrations/email_import.rs`: (1)
schema.org JSON-LD (`FlightReservation` / `TrainReservation` / `BusReservation`) from
`<script type="application/ld+json">` in HTML bodies, (2) ICS attachment extraction
(reuses `parse_ics`), (3) regex heuristics (IATA route pairs + flight numbers + dates).
Per-user inbound email tokens (`db/inbound_email_tokens.rs`, migration 020) provide
user identification — the webhook extracts the token from the recipient address (format
`import-{token}@domain`), hashes it, and resolves the owning user. Routes:
`POST /email/webhook` (public, SendGrid/Mailgun multipart), `POST /email/manual`
(authenticated, raw MIME), `POST /email/tokens` (generate address),
`POST /email/tokens/delete`. Journey creation reuses `create_journeys` from ICS import
(same dedup: date + origin + dest). Settings page shows "Email Import" section under
Data Sources with address generation and how-it-works instructions. Covered by 14 unit
tests (parser + route handler + token resolution + idempotent reimport).

#### 2.3 ICS calendar ingest

**Change:** we *produce* an ICS feed (`src/server/routes/feed.rs`) but don't *consume*
one. Add optional ingest of a user-supplied ICS URL to auto-create journeys, matching
Flighty's calendar auto-import.

**Effort:** Medium. **Impact:** Medium.

**✅ Done.** Settings → "Calendar Import": paste an `http(s)`/`webcal://` ICS URL, POST to
`/import/ics` (`routes/ics_import.rs`) fetches it (10s timeout, 5 MB cap) and creates
journeys. The pure parser `integrations/ics_import.rs::parse_ics` round-trips this app's own
feed (route from `LOCATION`, `Type:`/`Carrier:` from `DESCRIPTION`) and best-effort-parses
other calendars (any `A → B` route + start date; infers air for IATA pairs, else transport),
skipping non-travel events. Creation goes through `CreateManual` (so air gets airport-coord
resolution) and dedups against existing hops by (date, origin, dest) so re-imports are
idempotent. Covered by parser unit tests, a `normalize_url` test, and a create/dedup test.
Note: fetching a user-supplied URL is an SSRF surface — acceptable for self-hosted
single-tenant use; a multi-tenant deployment should block private/loopback ranges.

### Phase 3 — Social & live presence

#### 3.1 Live single-flight sharing ("Friends")

**Change:** extend the share-token model to share a *live* single flight (not just static
stats) with someone without login.

**Touch points:** `src/db/share_tokens.rs`, `src/server/routes/share.rs`, new public view.

**Effort:** Medium. **Impact:** Medium.

**✅ Done.** Per-journey share tokens in `db/journey_share_tokens.rs` (migration 021) allow
generating a public link per hop. `POST /journeys/{id}/share` (authenticated) creates a
token with 24-hour auto-expiry past the journey's departure. `GET /share/journey/{token}`
renders a stripped-down public page (`pages/shared_journey.rs`) showing route, travel type,
live status badge, delay, gate/terminal/platform chips, aircraft reg, and a map — no navbar,
no edit controls, no login required. `GET /share/journey/{token}/enrichments` returns a JSON
array matching the authenticated enrichment endpoint shape for client polling.
`static/js/shared-journey-poll.js` polls every 60s and updates the status badge in place.
The journey detail page shows a "Share" form to generate links, lists active shares with
copy/revoke controls. Expired tokens return 404; `DeleteExpired` is available for periodic
cleanup. Covered by 10 tests (DB CRUD, public render, expired/invalid 404, enrichment JSON,
authenticated creation, static asset).

#### 3.2 Delay reasons + trend

**Change:** surface *why* a flight is delayed (METAR/TAF weather codes, ATC) and a
worsening/improving trend. Requires a new weather/ATC data feed integration.

**Effort:** High (new data source). **Impact:** Medium.

## Explicitly out of scope

These are native-only or data-moat features where a self-hosted web app cannot realistically
compete, and they are not where our differentiation lies:

- **Native iOS Live Activities / lock-screen** — partially covered by rich push + the
  live-flight PWA view (1.2).
- **CarPlay / wearables** — partially covered by PWA home-screen shortcuts + a compact
  "next flight" view.
- **Pilot-grade FAA SWIM / Eurocontrol feeds** — expensive licensing.
- **Airport ground radar, taxi times, baggage-belt intelligence** — data-feed dependent
  and outside our value proposition.

## Suggested sequencing

1. ~~**1.1 Push on status delta**~~ — ✅ shipped (see Phase 1.1 above).
2. ~~**1.2 Upcoming + live flight view**~~ — ✅ shipped (see Phase 1.2 above).
3. ~~**2.1 Year-in-review**~~ — ✅ shipped (see Phase 2.1 above).
4. **1.3 Inbound-aircraft prediction** — 🟡 foundation shipped (tail-number capture); the
   live prediction lookup awaits AirLabs-key validation (see Phase 1.3 above).
5. ~~**2.3 ICS import**~~ ✅ shipped; ~~**2.2 email import**~~ ✅ shipped;
   ~~**3.1 live single-flight sharing**~~ ✅ shipped; **3.2 delay reasons/trend** remains.

## Design & Layout

Findings from a live walkthrough of the running app (logged in as the seed user, screens
captured at 1440×900: dashboard, stats, trips, settings, journeys list, journey detail,
trip detail, add-journey).

**Verdict:** the visual language was already strong — coherent dark theme, tasteful single
accent, colorblind-safe travel palette, carrier logos, `LHR → DUB` route arrows, and a
dashboard right-rail ("Upcoming" / "Past Journeys" with relative dates) that already
mirrors Flighty. The gaps were **layout density and consistency**, not aesthetics — all six
issues below have now been addressed (P0–P2).

### Issues observed — all resolved ✅

1. ~~**Dashboard header eats the page.**~~ Fixed (P1): four headline cards + "more stats" /
   "more filters" disclosures keep the map near the top.
2. ~~**Map theming inconsistent across four implementations.**~~ Fixed (P0): all four maps
   share `static/js/map-core.js` and the dark basemap; journey/trip maps draw type-coloured
   great-circle arcs.
3. ~~**Button styling inconsistent.**~~ Fixed (P0): every button is one of
   primary/secondary/success/danger; orphan `.btn-warning` removed, `.btn-lg` defined,
   `.btn-block` → `.btn-full`.
4. ~~**Add-Journey form sparse and mis-validates.**~~ Fixed (P0 + P2): `:user-invalid` stops
   premature red borders; the form is now grouped sections in a two-column grid (and a latent
   type-fields show/hide bug was fixed).
5. ~~**Stat cards flat.**~~ Fixed (P1): `--text-3xl/4xl` display scale + travel-type icons;
   hero figures enlarged.
6. ~~**Trip cards uniform.**~~ Fixed (P2): travel-type emoji glyphs + a left-border accent by
   dominant mode.

### Prioritised changes

| Priority | Change                                                          | Effort | Status |
| -------- | --------------------------------------------------------------- | ------ | ------ |
| **P0**   | Consolidate the 4 map scripts into one shared module; dark everywhere | Low    | ✅ done |
| **P0**   | Fix Add-Journey premature validation                            | Low    | ✅ done |
| **P0**   | Consolidate button styles into primary/secondary/success/danger | Low    | ✅ done |
| **P1**   | Slim dashboard header — ~4 headline stats + collapsible filters | Medium | ✅ done |
| **P1**   | Add display type scale (`--text-3xl/4xl`) + icons to stat cards | Medium | ✅ done |
| **P2**   | 2-column Add-Journey layout, grouped sections                   | Medium | ✅ done |
| **P2**   | Richer trip cards (route glyph / mini-map thumbnail)            | Medium | ✅ done |

**Progress log:**

- **P0 maps** — extracted `static/js/map-core.js` (ES module): shared dark CARTO basemap,
  map defaults, `arcPoints`/`haversineKm`, `escapeHtml`, `cssVar`, and travel-type palette.
  `map.js`, `stats-map.js`, `journey-map.js`, `trip-map.js` now import it. The journey- and
  trip-detail maps are dark and draw type-coloured great-circle arcs (were light OSM + flat
  blue polylines). This satisfies the Tier A "shared map module" prerequisite — MapLibre +
  Protomaps + deck.gl (Tiers B/C) remain roadmapped.
- **P0 validation** — `forms.css` now uses `:user-invalid` so required fields (notably
  `date`, which has no placeholder) only flag after interaction.
- **P0 buttons** — every actionable button now carries exactly one of
  primary/secondary/success/danger; removed the orphan `.btn-warning`, defined the
  referenced-but-missing `.btn-lg`, fixed `.btn-block` → `.btn-full`.
- **P1 dashboard header** — collapsed to four headline stat cards with a "more stats"
  disclosure; the ~10-control filter bar is now a collapsible `<details>` panel so the map
  sits near the top of the viewport.
- **P1 stat cards** — added `--text-3xl`/`--text-4xl` display type scale and travel-type
  icons; hero figures use the larger scale.
- **P2 Add-Journey** — regrouped into "Route" / "Details" / "Cost & Loyalty" sections with a
  two-column `.form-row`/`.form-grid` (collapses to one column under 640px); notes textareas
  span full width. Also fixed a latent bug — the type-specific field blocks were toggled by
  clearing an inline `display` against a `.type-fields{display:none}` rule (so they never
  showed); now toggled via an `is-active` class, and the IATA uppercasing moved from inline
  `style.textTransform` to a `.uppercase-input` class.
- **P2 trip cards** — `trips` query now aggregates `GROUP_CONCAT(DISTINCT travel_type)`; cards
  show travel-type emoji glyphs and a left-border accent coloured by dominant mode
  (air→rail→boat→transport priority), turning the uniform grid into scannable cards.

### Map & visualisation upgrade

Applies to **all four map surfaces** — dashboard, stats, trip detail, and journey detail.
They feel plain because they use flat **raster** basemaps (CARTO `dark_all` / light OSM)
with hand-computed flat `L.polyline` great-circle arcs, and the logic is duplicated across
`map.js`, `stats-map.js`, `trip-map.js`, and `journey-map.js`.

**Prerequisite — shared map module. ✅ Done.** The basemap, map defaults, and great-circle
arc maths were extracted into `static/js/map-core.js`, which all four scripts import — so a
single styling change now lands on every map. Three tiers build on top of it:

- **Tier A — stay on Leaflet (low effort). ✅ Done.** Superseded by Tier B+C — all
  four maps now use MapLibre GL JS + Protomaps vector tiles with a satellite layer
  toggle.
- **Tier B — vector basemap (the real modernisation). ✅ Done.** All four map surfaces
  migrated from Leaflet to **MapLibre GL JS v5** with **Protomaps** dark vector tiles
  via PMTiles protocol (self-hosted, no API key). Globe projection enabled on all maps
  via `map.setProjection({ type: 'globe' })`. Navigation controls include pitch
  visualisation, a globe toggle (GlobeControl), and a satellite imagery toggle (Esri
  World Imagery as a raster layer under the vector labels).
- **Tier C — GPU arcs / globe (the "wow"). ✅ Done.** **deck.gl** `ArcLayer` (via
  `MapboxOverlay`) renders GPU-accelerated great-circle arcs with 3D height on the
  dashboard, journey-detail, and trip-detail maps. Arcs are coloured by travel type,
  width scales with route frequency on the dashboard, and clicking an arc shows the
  route popup. The stats choropleth uses MapLibre's native GeoJSON fill/line layers
  with data-driven expressions (no deck.gl needed).

**Recommendation:** All three tiers are shipped — Protomaps dark vector basemap, MapLibre
globe projection, deck.gl GPU arcs, satellite toggle. The maps are now fully
self-hostable (PMTiles, no API keys) and visually distinctive.

## References

- [Flighty.com](https://flighty.com/)
- [Flighty Passport](https://flighty.com/passport)
- [The Points Guy — Everything about Flighty](https://thepointsguy.com/travel-gear/everything-you-need-to-know-about-flighty-app/)
- [Upgraded Points — 2025 improvements](https://upgradedpoints.com/news/flighty-improvements-2025/)
- [Flighty — Wikipedia](https://en.wikipedia.org/wiki/Flighty)
