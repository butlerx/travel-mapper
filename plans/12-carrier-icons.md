# 12 — Carrier Icons

## What

Display airline, train operator, and ferry provider logos/icons alongside journeys in the UI.

## Approach

### Airlines
- **IATA codes already stored** in hop data — use as lookup key
- **Icon source**: Use a public CDN like `https://images.kiwi.com/airlines/64/{IATA}.png` or bundle a curated set
- Fallback: Generic plane icon when carrier not found

### Train Operators
- Match by carrier name string (less standardized than airlines)
- Curate a small icon set for major operators (Irish Rail, Deutsche Bahn, SNCF, Amtrak, etc.)
- Fallback: Generic train icon

### Ferry/Boat Operators
- Same approach as trains — match by name, curated set
- Fallback: Generic boat icon

### Implementation
1. **Carrier icon component**: Leptos component that takes carrier code/name + travel type, returns `<img>` or fallback SVG
2. **Icon resolution**: Try IATA code CDN first (flights), then local lookup table, then generic icon
3. **Display locations**: Journey list cards, journey detail header, trip detail journey list
4. **Caching**: If using external CDN, service worker caches icons

## Dependencies

- None — purely additive UI enhancement

## Files

- `src/server/components/` — new `carrier_icon.rs` component
- `src/server/pages/dashboard.rs` — use carrier icon in journey cards
- `src/server/pages/hop_detail.rs` — use in detail header
- `src/server/pages/trip_detail.rs` — use in trip journey list
- `static/sw.js` — cache icon CDN responses (if external)
- Optionally `static/icons/` for bundled fallback icons

## Acceptance Criteria

- [ ] Airline logos display for flights with known IATA codes
- [ ] Train operator icons display for major operators
- [ ] Ferry operator icons display where available
- [ ] Fallback generic icons for unknown carriers
- [ ] Icons appear on journey list, detail page, and trip views
- [ ] No broken images — fallback always works
- [ ] All existing tests pass
- [ ] `mise run lint` clean
