# Requirement

## Background

DeepSeek and Kimi OpenAI-compatible responses may report cache and reasoning usage through several equivalent fields. SunCode currently retains only a subset, and its leading host-environment message changes on every provider call because it includes the current second. This prevents reliable cache diagnostics and can invalidate exact prompt-prefix matching.

## Goals

- Normalize common OpenAI-compatible cache-hit, cache-miss, cache-write, and reasoning-token fields.
- Persist the normalized call usage in `session_call.usage_json`.
- Keep the leading host-environment prompt stable throughout a session.

## Non-goals

- Persist raw provider response payloads.
- Change cumulative session billing usage or the SQLite table schema.
- Add provider-specific adapters or explicit cache-control request options.

## Requirements

- Nested standard fields take precedence over compatible top-level aliases.
- Missing optional usage remains `null`; an explicit zero remains zero.
- Call usage includes normalized `cache_read_tokens`, `cache_miss_tokens`, `cache_write_tokens`, and `reasoning_tokens`.
- The host-environment message uses the stable session start timestamp instead of the current call time.

## Edge cases

- Providers may return both nested and legacy aliases with different values.
- Providers may omit cache-miss, cache-write, or reasoning details.
- A recovered legacy continuation may not contain the session start timestamp.

## Acceptance criteria

- DeepSeek/Kimi example usage shapes normalize without losing reported cache or reasoning counts.
- Normalized optional usage reaches `session_call.usage_json`.
- Repeated provider calls in one session have an identical leading host-environment message.
- Focused Rust tests and workspace validation pass.

## Open questions

- None.
