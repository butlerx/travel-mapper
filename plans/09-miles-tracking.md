# 09 — Frequent Flyer / Miles Tracking

## What

Track loyalty program memberships and miles/points earned per journey, with totals by program.

## Approach

1. **Schema**: New `loyalty_programs` table (user_id, program_name, member_number) and `miles_earned` column or table linked to hops
2. **Per-journey entry**: Optional loyalty program + miles on journey create/edit forms
3. **Auto-calculation**: Estimate miles from great-circle distance for flights if not manually entered
4. **Stats integration**: Miles totals on stats page, grouped by program
5. **Settings**: Manage loyalty program memberships

## Dependencies

- Stats page (done)
- Journey detail/edit (done)

## Files

- `migrations/` — new `loyalty_programs` table, add miles fields to hops or new junction table
- `src/db/` — new `loyalty_programs.rs`, update hop queries
- `src/server/routes/hops.rs` — accept miles/program fields
- `src/server/pages/add_hop.rs` — loyalty program selector
- `src/server/pages/hop_detail.rs` — display/edit miles
- `src/server/pages/stats.rs` — miles summary section
- `src/server/pages/settings.rs` — loyalty program management

## Acceptance Criteria

- [ ] User can add loyalty programs in settings
- [ ] Journey create/edit form has optional program + miles fields
- [ ] Miles auto-estimated from distance for flights when not entered
- [ ] Stats page shows total miles by program
- [ ] All existing tests pass
- [ ] `mise run lint` clean
- [ ] `.sqlx/` cache regenerated
