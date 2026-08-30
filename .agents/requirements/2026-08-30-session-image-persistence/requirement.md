# Session Image Persistence Requirement

- Date: 2026-08-30
- Status: Complete

## Context

The Avalonia composer already supports local image preview, removal, and a three-image limit, but those attachments are still transient control state. The current `submit_turn` SDK method is text-only and must stay text-only for now, so image attachments need their own durable session-owned placeholder path.

## Requirements

- Keep `submit_turn` unchanged: uploaded images are not sent to model providers and are not stored in session messages or provider calls.
- Persist uploaded session images through the Rust-owned SDK and SQLite boundary rather than through Avalonia-local files or client-owned databases.
- Default image storage to `~/.suncode/data/images`, with one per-session directory and one file per image at `~/.suncode/data/images/{sessionId}/{imageId}.{ext}`.
- Add a Settings entry that lets users change the image storage directory, following the same global-setting pattern as log-directory configuration.
- Add a SQLite `session_image` table that stores the image ID, owning session ID, image source kind, original path when the image came from a file, and a database-owned thumbnail payload so the desktop can restore previews without loading the full file.
- Preserve enough durable storage-path information for previously uploaded images to remain readable after the global image directory setting changes.
- Support both file uploads and clipboard-image uploads in the persisted model. Clipboard uploads must be marked as clipboard-sourced even when they are saved to a regular image file on disk.
- Restore persisted session images when a session is reopened or reselected, and allow users to remove them from the composer placeholder strip.

## Non-goals

- Sending uploaded images through `submit_turn` or provider requests.
- Storing original full image bytes inside `session_message`, `session_call`, or `session_tool_use`.
- Introducing a general SQLite migration runner.
