# Test Plan

## Rust

- Open a fresh in-memory store and confirm `image_directory` is seeded and typed.
- Add a session image, verify the file is written, verify the row is returned, then remove it and verify cleanup.
- Reopen an existing current database missing `session_image` and confirm the table is added during initialization.

## Avalonia

- Upload an image from file and confirm it appears in the composer strip.
- Paste an image from the clipboard and confirm it appears in the composer strip.
- Reselect the same session and confirm persisted images reload.
- Remove a persisted image and confirm it disappears without sending a turn.
- Save a custom image directory in Settings and confirm later uploads use that location.
