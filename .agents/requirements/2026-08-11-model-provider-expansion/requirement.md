# Requirement

## Background

The Phase 1 Qt desktop client currently supports DeepSeek as the only built-in model provider. The settings page, runtime credential store, and model registry need to expand to support additional OpenAI-compatible providers without changing the embedded runtime boundary.

## Goals

- Support DeepSeek, Zhipu GLM, and OpenAI as first-class model providers.
- Keep provider credentials in Rust-owned plaintext SQLite secret records.
- Keep the model selector populated from the local runtime.
- Keep the settings page organized by provider with one detail page per provider.

## Non-goals

- Hosted identity, multi-tenancy, or cloud control plane changes.
- New provider families beyond DeepSeek, Zhipu GLM, and OpenAI.
- Protocol generation or database schema changes.

## Requirements

- Preserve existing DeepSeek support.
- Add runtime support for Zhipu GLM and OpenAI using the same local authenticated flow.
- Expose per-provider credential state to the Qt client.
- Keep the composer model selector and session creation path working with any advertised model.
- Keep the settings tree grouped by provider with provider-specific credential pages.

## Acceptance criteria

- The runtime advertises models for DeepSeek, Zhipu GLM, and OpenAI when configured.
- The Qt settings page can store and remove credentials for each supported provider.
- The composer disables sending when the selected model's provider is not configured.
- The project builds and the desktop client launches successfully.
