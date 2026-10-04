# Design System Review Browser

**Status:** Implemented; specification and review tooling only

`design-system/` is a private React/Vite application that serves as the executable review surface and resource catalog for root `DESIGN.md`. `DESIGN.md` is the written authority. This project holds the specimens, tokens, and assets that production UI is compared against. React, Vite, and Node.js are used only to build this tool. They are not runtime dependencies of the Avalonia desktop client or the embedded Rust agent, and this is not a Web client.

## What it contains

- One entry point (`index.html` and `src/main.jsx`) with hash routes, so `npm run build` produces a static `dist/index.html` that works from any subpath or opened directly. `dist/` and `node_modules/` are ignored.
- Five primary layers, switched by name from the upper-right: Core, Components, Platforms, Projects, and Agents. Each layer opens a card index. The left sidebar is a contextual, recursive tree for the active layer only. Every child entry has its own stable route, and parent pages stay as indexes.
- Universal component specimens under `src/components/universal/<component>/`. Each folder owns its specimen, its styles, and an `index.js` export. Category pages (Actions, Fields, Selection, Surfaces, Overlays, Navigation, Feedback, Data, Markdown) live under `modules/`.
- Platforms for Desktop, Mobile, TUI, and Web. Only Desktop is implemented for Phase 1. The other three are labeled deferred and describe their boundaries without inventing component libraries.
- Desktop project specimens under `src/projects/desktop/` for ProjectHub, Workspace, Settings, About, and DialogWindow. Workspace has a full window overview plus standalone routes for Sessions, Explorer, Conversation, Review, Source control, Provider trace, Tool activity, Editor, and Child sessions. Settings specimens include provider overview and detail pages, Network certificate and proxy controls, Language servers, and Appearance with the interface-language row.
- Agents pages that record the General agent's prompt surfaces, along with their scope, owner, and implementation status.
- Token sources in `src/styles/tokens/`, source-imported assets in `src/assets/` (cataloged in its README), and browser-direct files such as the favicon in `public/assets/`.

## How it is used

- Before any UI or interaction change, review the matching route and its tokens. If a state or component is missing, add the specimen in the same change. Production Avalonia views must match the specimens. Any intentional deviation must be recorded in both `DESIGN.md` and the specimen.
- Avalonia resources in `apps/desktop-avalonia/App.axaml` and `Styles/Application/` hand-map the same semantic token names and values. Nothing is generated from this project.
- Specimens show light and dark themes with the same semantics. The browser opens in light theme and stores the reviewer's theme choice locally in the browser only.
- Status labels stay honest: implemented, review reference, reserved, or deferred.

`design-system/README.md`, `CONTRIBUTING.md`, and `AGENTS.md` define the local rules for file layout and contributions. Verify with `npm run build`, plus `npm run format:check`, inside `design-system/`.
