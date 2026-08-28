# Universal Components

This layer owns cross-platform interaction primitives and their review states. Each component folder owns its React specimen and a stable `index.js` export. `react/UniversalComponentsPage.jsx` is only the catalog composition entry: it imports those modules, groups them into review sections, and must not duplicate component implementation.

The Avalonia client remains the runtime owner of presentation. This directory must not become a second production component library.
