# 11 — Cost Tracking

## What

Track ticket prices per journey with currency support and spending analytics.

## Approach

1. **Schema**: Add `cost_amount` (real, nullable) and `cost_currency` (text, nullable, ISO 4217) columns to `hops` table
2. **Journey forms**: Optional cost + currency fields on create/edit
3. **Display**: Show cost on journey detail page and journey list
4. **Stats**: Spending analytics on stats page — total by time period, by carrier, by route, by travel type
5. **Currency**: Store as-entered (no conversion); display with currency symbol; optionally add conversion later

## Dependencies

- Journey detail/edit (done)
- Stats page (done)

## Files

- `migrations/` — add `cost_amount`, `cost_currency` to `hops`
- `src/db/hops/` — update create/update queries, add cost to row mapping
- `src/server/routes/hops.rs` — accept cost fields
- `src/server/pages/add_hop.rs` — cost input fields
- `src/server/pages/hop_detail.rs` — display cost
- `src/server/pages/stats.rs` — spending summary section

## Acceptance Criteria

- [ ] User can enter cost + currency when creating/editing a journey
- [ ] Cost displayed on journey detail page
- [ ] Stats page shows spending summaries
- [ ] Currency stored as ISO 4217 code
- [ ] Null cost is handled gracefully (most journeys won't have cost)
- [ ] All existing tests pass
- [ ] `mise run lint` clean
- [ ] `.sqlx/` cache regenerated
