# Desktop Settings Review Surface

Settings is organized into:

- `components/` for small settings controls and row primitives.
- `sections/` for top-level settings panels and their list/state orchestration, including the Remote Server connection and mobile pairing flow.
- `dialogs/` for editor windows and consequential settings flows.
- `data/` for provider, guide, MCP, and language-server catalog fixtures.
- `styles/` for settings-owned visual rules.

`index.jsx` is the stable route entrypoint. `SettingsPage.jsx` owns only page state, navigation, and section selection; section implementations, dialogs, and fixtures live under their focused directories. Keep route imports pointed at `index.jsx` so internal file moves do not spread through the application shell.

The Remote Server specimen covers disconnected input, pairing-code validation, connected status, mobile QR presentation, QR renewal, and disconnect. Its pairing QR is a visual placeholder and does not encode a live credential.
