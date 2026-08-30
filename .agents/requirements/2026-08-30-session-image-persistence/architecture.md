# Architecture Notes

## Ownership

- Avalonia owns file-picker and clipboard interaction plus thumbnail generation for user-selected images.
- Rust owns the canonical image directory resolution, on-disk file writes, SQLite rows, and session-image listing/removal APIs.
- SQLite keeps image metadata and thumbnails; the original image files remain on disk under the configured image directory.

## Data model

- Add global `image_directory` configuration. Empty string means `<data directory>/data/images`.
- Add `session_image` as a session-owned table with:
  - `image_id`
  - `session_id`
  - `display_name`
  - `source_kind`
  - nullable `original_path`
  - durable `storage_path`
  - `thumbnail_base64`
  - `created_at`

## SDK surface

- `list_session_images(session_id)` returns the durable placeholder images for one session.
- `add_session_image(session_id, payload)` writes the image file, stores metadata, and returns the created record.
- `remove_session_image(session_id, image_id)` removes one persisted placeholder image and best-effort deletes its file.

## UI behavior

- Composer images become session-owned placeholder state rather than one-submit transient state.
- Text submission no longer clears uploaded images automatically because they are intentionally outside the text-only turn payload.
