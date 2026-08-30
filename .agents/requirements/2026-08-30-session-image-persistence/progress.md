# Progress

- Confirmed the current composer attachments are local-only control state and are cleared on text submission.
- Chose an SDK shape that keeps `submit_turn` text-only and routes images through separate session-image methods.
- Implemented durable session-image storage, metadata, thumbnails, and Settings support.
- Restored session images into the Avalonia composer and added clipboard image paste support.
