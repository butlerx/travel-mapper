# 10 — Photo Attachments

## What

Attach photos to journeys — boarding passes, scenic views, tickets — with a gallery view on the detail page.

## Approach

1. **Storage**: Local filesystem (configurable path via env var) with `{user_id}/{hop_id}/{uuid}.{ext}` structure
2. **Schema**: New `attachments` table (id, hop_id, user_id, filename, content_type, size_bytes, created_at)
3. **Upload endpoint**: `POST /journeys/{id}/attachments` — multipart file upload
4. **Serve endpoint**: `GET /attachments/{id}` — serve file with proper content-type
5. **Delete endpoint**: `DELETE /journeys/{id}/attachments/{attachment_id}`
6. **Detail page**: Gallery/grid view of attached photos below journey details
7. **Limits**: Max file size (configurable), max attachments per journey, image-only MIME type validation

## Dependencies

- Journey detail page (done)

## Files

- `migrations/` — new `attachments` table
- `src/db/` — new `attachments.rs`
- `src/server/routes/` — new `attachments.rs` (upload, serve, delete)
- `src/server/state.rs` — register attachment routes
- `src/server/pages/hop_detail.rs` — photo gallery component
- `src/server.rs` — storage path config

## Acceptance Criteria

- [ ] User can upload photos to a journey
- [ ] Photos display in a gallery on the journey detail page
- [ ] Photos can be deleted
- [ ] File size and type limits enforced
- [ ] Files stored on disk, not in DB
- [ ] All existing tests pass
- [ ] `mise run lint` clean
- [ ] `.sqlx/` cache regenerated
