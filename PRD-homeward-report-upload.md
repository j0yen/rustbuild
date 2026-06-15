# PRD: homeward-report-upload — direct photo upload for lost-pet reports

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
build_version_bump: minor

## TL;DR

Add `POST /uploads` to homeward-reportd so owners can upload a photo directly
(multipart/form-data) instead of supplying an external hotlink URL. The server
strips EXIF, saves the file under `$HW_UPLOAD_DIR` (default
`~/.local/share/homeward/uploads/`), and returns a JSON `{ "url": "..." }` that
the caller can pass as `photo_url` in `POST /reports`. No photo = no visual match
from the MatchWatcher; this removes the last friction point blocking a real user
from submitting a report.

## Why this exists (Phase 6 reflect — homeward v0.24.0 shipped)

homeward v0.24.0 closes the full match-and-notify loop:

```
POST /reports (+ photo_url)
  → MatchWatcher polls embed sidecar
    → MatchAlert → webhook fire
      → GET /reports/:id/matches
```

But `photo_url` is still a **hotlink to an external URL** the owner must
already have. In practice:

- Owners have photos on their phone; they do not have a CDN.
- The web UI's drag-and-drop is for one-off searches, not for report submission.
- Until owners can upload a file during submission, the match loop is
  unreachable for the most common user.

## Acceptance tests

### AC1 — Upload returns a usable URL
`POST /uploads` with `Content-Type: multipart/form-data` containing a JPEG
`file` field → HTTP 200 `{ "url": "/uploads/<filename>" }`. The URL resolves
via `GET /uploads/<filename>` (served as `image/jpeg` from the upload dir).

### AC2 — EXIF stripped before storage
The stored file must not contain GPS or author EXIF metadata. After `POST /uploads`,
a `GET /uploads/<filename>` response image has no GPS tags (verified via
`exiftool -GPS* -n` or the `kamadak-exif` crate reader returning no GPS fields).

### AC3 — Size limit enforced
`POST /uploads` with a body > 10 MB → HTTP 413. The default limit is
`HW_UPLOAD_MAX_BYTES` (default 10_485_760).

### AC4 — Non-image rejected
`POST /uploads` with a `text/plain` payload → HTTP 415. Only `image/jpeg`,
`image/png`, and `image/webp` are accepted.

### AC5 — Upload URL flows into POST /reports
`POST /reports` with `photo_url` set to the URL returned by AC1 → HTTP 201.
`GET /reports/:id` returns the report with `photo_url` matching the upload URL.

### AC6 — Upload dir configurable
When `HW_UPLOAD_DIR=/tmp/hw-test-uploads` is set, files land there, not in
the default dir. The dir is created on startup if absent.

## Scope

**In scope:**
- New `homeward-report` handler `handle_upload` behind `POST /uploads`
- Static file serving for `GET /uploads/<filename>` (axum `ServeDir` or
  manual handler)
- EXIF stripping with the `kamadak-exif` crate (remove-and-rewrite strategy:
  re-encode the image without GPS/author tags using the `image` crate)
- `HW_UPLOAD_MAX_BYTES` env var, default 10 MiB
- `HW_UPLOAD_DIR` env var, default `~/.local/share/homeward/uploads/`
- Unit tests for all six ACs

**Out of scope:**
- Web UI update (follow-on PRD: homeward-report-form)
- Upload auth / API keys (the server is localhost-only for now)
- Permanent storage / deduplication by content hash (v1: just store by random UUID)
- Virus scanning

## Implementation notes

- Use `axum::extract::Multipart` for the file upload.
- Filename: `<ulid>.jpg` (always output JPEG after re-encode; avoids serving
  content-type confusion).
- EXIF stripping: decode with `image::load_from_memory`, re-encode to JPEG with
  `image::codecs::jpeg::JpegEncoder` — this drops all EXIF by default because
  `image` does not carry EXIF through its re-encode path.
- `ServeDir` from `tower-http` serves the uploads directory as static files at
  `/uploads/*`.
- New deps: `image`, `axum::extract::Multipart`, `tower-http ServeDir` (already
  present from `TraceLayer` dep, just enable the `fs` feature).
- Route all cargo invocations through cloudbuild.
