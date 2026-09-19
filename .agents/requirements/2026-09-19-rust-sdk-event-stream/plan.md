# Implementation Plan

1. [x] Confirm scope and affected boundaries.
2. [x] Add typed core events and a session-scoped bounded event hub.
3. [x] Replace the Rust callback subscription with a typed stream.
4. [x] Move callback and JSON adaptation into `sdks/c` without changing ABI.
5. [x] Add focused isolation, lag, close, and compatibility tests.
6. [x] Run broader Rust and Avalonia verification appropriate to risk.
7. [x] Update durable project knowledge.
