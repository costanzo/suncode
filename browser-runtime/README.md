# Bundled Browser Runtime

This directory owns the first-party Playwright Browser Use worker and the immutable runtime version lock used by release packaging.

Production packages contain exactly one target-specific Node.js executable, the production `playwright` and `playwright-core` packages, Playwright Chromium revision 1187, Playwright's required FFmpeg helper revision 1011, and `worker/index.mjs`. SunCode never installs npm packages or downloads a browser after packaging.

The worker is not an agent or extension host. Rust launches it with a filtered environment and communicates through the framed protocol in `contracts/browser-worker.md`.

## Development

```sh
cd browser-runtime/worker
npm ci
```

Chromium is intentionally not downloaded by `npm ci`. Release packaging sets a target-specific browser path and runs the pinned Playwright installer with `--no-shell chromium`.
