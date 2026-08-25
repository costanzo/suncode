# Architecture

## Current state

Avalonia owns runtime presentation in `apps/desktop-avalonia/`. `DESIGN.md` describes the product language, while the runtime theme dictionaries and view-local styles implement it.

## Proposed design

The root `design/` directory is the review boundary for client visual language. `tokens.css` defines shared names and both theme values. `dark.html` and `light.html` consume the same component markup so reviewers can compare semantic changes between themes. `design/assets/` owns the reusable design asset catalog.

## Boundaries and dependencies

The review pages are static HTML and CSS only. They do not become a production web client and do not change the Rust SDK or protocol boundaries. Avalonia remains the only Phase 1 production client.

## Data and control flow

Reviewer -> `index.html` -> `dark.html` or `light.html` -> `tokens.css` and `design/assets/`.

`index.html` is the desktop-first shared entry point. It presents cross-theme foundations, semantic color roles, desktop shell anatomy, review rules, and links to the theme-specific component pages. It is a static review artifact, not a production web client.

Runtime client -> Avalonia resource dictionaries and semantic view resources, kept aligned with the design review tokens.

## Security and failure handling

No network resources, executable scripts, credentials, or generated runtime state are used. Missing optional images must not block design page rendering.

## Compatibility and migration

No migration is needed. Existing Avalonia assets remain in the application packaging boundary; approved design assets are cataloged under `design/assets/` for review and future client packaging.

## Risks and rollback

The primary risk is token drift between `tokens.css` and Avalonia resources. Review changes can be rolled back by reverting the design directory and documentation changes without affecting runtime persistence or SDK behavior.

## Open questions

The project has not yet selected a cross-platform font packaging strategy. The pages use the documented fallback stacks until that decision is made.
