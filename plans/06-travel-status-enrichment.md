# 06 — Travel Status Enrichment

## What

Enrich flight and train records with live/historical status data from external APIs — delays, cancellations, gate/platform info, actual arrival times.

## Approach

### Flights
- **API options**: AviationStack (free tier), FlightAware AeroAPI, or FlightRadar24
- Match by flight number + date
- Store enriched data: actual departure/arrival, delay minutes, status (on time/delayed/cancelled), gate, terminal

### Trains
- **API options**: Varies by region — Irish Rail API, Deutsche Bahn, National Rail (UK), Amtrak
- Match by train number/service + date
- Store: actual departure/arrival, delay, platform, status

### Shared
1. **Config**: Per-provider API keys in env vars; provider selection in settings
2. **Enrichment table**: New `status_enrichments` table (hop_id, provider, fetched_at, status_json)
3. **Worker integration**: Enrich after sync, or on-demand via button
4. **UI**: Show status badges (on time/delayed/cancelled) on journey list and detail pages
5. **Rate limiting**: Cache responses, respect API rate limits

## Dependencies

- Core data model (done)
- Item 12 (Carrier Icons) is complementary — carrier logos alongside status

## Files

- `migrations/` — new `status_enrichments` table
- `src/db/` — new `status_enrichments.rs`
- `src/integrations/` — new provider modules (e.g., `aviationstack.rs`, `rail_api.rs`)
- `src/worker.rs` — enrichment step after sync
- `src/server/pages/hop_detail.rs` — display status
- `src/server/pages/dashboard.rs` — status badges on journey cards
- `src/server.rs` — API key config

## Acceptance Criteria

- [ ] Flight records are enriched with live status data
- [ ] Train records are enriched where API coverage exists
- [ ] Status shown on journey detail page (delay, actual times, platform/gate)
- [ ] Status badges on journey list
- [ ] API failures are graceful (enrichment is optional, never blocks display)
- [ ] Rate limits respected
- [ ] All existing tests pass
- [ ] `mise run lint` clean
