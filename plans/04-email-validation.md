# 04 — Email Validation

## What

Add email addresses to user accounts with a verification flow — send a token via email, user clicks confirmation link, account is marked verified.

## Approach

1. **Schema**: Add `email` (nullable text) and `email_verified_at` (nullable timestamp) columns to `users` table via migration
2. **Registration**: Accept optional `email` field during registration
3. **Verification token**: Generate a random token, store in a `email_verifications` table with expiry
4. **Send email**: Use `lettre` crate for SMTP or a simple HTTP-based email provider
5. **Confirm endpoint**: `GET /auth/verify-email?token=...` validates token, sets `email_verified_at`
6. **Settings page**: Allow adding/changing email, trigger re-verification
7. **Env vars**: SMTP config (`SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `FROM_EMAIL`)

## Dependencies

- Item 3 (Registration Gate) is complementary but not blocking

## Files

- `migrations/` — new migration for `email`, `email_verified_at` columns + `email_verifications` table
- `src/db/users.rs` — update user queries to include email fields
- `src/db/` — new `email_verifications.rs` for token CRUD
- `src/server/routes/register.rs` — accept email field
- `src/server/routes/` — new `verify_email.rs` handler
- `src/server/pages/settings.rs` — email management UI
- `src/server.rs` — SMTP config in app state

## Acceptance Criteria

- [ ] User can register with optional email
- [ ] Verification email is sent with a confirmation link
- [ ] Clicking the link marks the account as verified
- [ ] Settings page shows email status and allows changes
- [ ] Expired tokens are rejected
- [ ] All existing tests pass
- [ ] `mise run lint` clean
- [ ] `.sqlx/` cache regenerated
